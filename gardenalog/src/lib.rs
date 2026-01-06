//! tracing subscriber for GARDENAs custom log format

use tracing_subscriber::layer::SubscriberExt as _;

/// Debug-format all values and store them in a hashmap.
struct CollectingVisitor<'a> {
    fields: &'a mut std::collections::HashMap<tracing::field::Field, String>,
}

impl<'a> CollectingVisitor<'a> {
    pub fn new(fields: &'a mut std::collections::HashMap<tracing::field::Field, String>) -> Self {
        Self { fields }
    }
}

impl tracing::field::Visit for CollectingVisitor<'_> {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn core::fmt::Debug) {
        self.fields.insert(field.clone(), format!("{value:?}"));
    }
}

/// A tracing Span extension which provides Debug-formatted values.
#[derive(Default)]
struct FormattedValues {
    fields: std::collections::HashMap<tracing::field::Field, String>,
}

/// Attaches [FormattedValues] to all spans
struct ValuesLayer;

impl<S> tracing_subscriber::Layer<S> for ValuesLayer
where
    S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    fn on_new_span(
        &self,
        attrs: &tracing::span::Attributes<'_>,
        id: &tracing::span::Id,
        ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let span = ctx.span(id).expect("Span not found, this is a bug");
        let mut extensions = span.extensions_mut();

        if extensions.get_mut::<FormattedValues>().is_none() {
            let mut values = FormattedValues::default();
            attrs
                .values()
                .record(&mut CollectingVisitor::new(&mut values.fields));
            extensions.insert(values);
        }
    }
}

/// Escape the `val` parameter according to PARAM-VALUE rule from RFC5424.
///
/// Source: <https://github.com/nocduro/syslog5424/blob/ecfe728e23df5e751d8bcdb9e070df6376e19c69/src/types.rs>
#[inline]
fn escape_val(val: &str) -> String {
    val.replace('\\', r"\\")
        .replace('"', r#"\""#)
        .replace(']', r"\]")
}

/// Remove invalid characters from `name`. Used for values in PARAM-NAME
/// and SD-ID from RFC5424. Removes `'=', ' ', ']', '"'`, and non-printable
/// ASCII characters. The filtered message is then truncated to 32 characters.
///
/// Source: <https://github.com/nocduro/syslog5424/blob/ecfe728e23df5e751d8bcdb9e070df6376e19c69/src/types.rs>
#[inline]
fn remove_invalid(name: &str) -> String {
    name.chars()
        .filter(char::is_ascii_graphic)
        .filter(|c| *c != '=')
        .filter(|c| *c != ' ')
        .filter(|c| *c != ']')
        .filter(|c| *c != '"')
        .take(32)
        .collect()
}

struct MyFormatter;

impl<S, N> tracing_subscriber::fmt::FormatEvent<S, N> for MyFormatter
where
    S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
    N: for<'a> tracing_subscriber::fmt::FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &tracing_subscriber::fmt::FmtContext<'_, S, N>,
        mut writer: tracing_subscriber::fmt::format::Writer<'_>,
        event: &tracing::Event<'_>,
    ) -> std::fmt::Result {
        let metadata = event.metadata();
        let level = match *metadata.level() {
            tracing::Level::ERROR => libc::LOG_ERR,
            tracing::Level::WARN => libc::LOG_WARNING,
            tracing::Level::INFO => libc::LOG_INFO,
            tracing::Level::DEBUG => libc::LOG_DEBUG,
            tracing::Level::TRACE => libc::LOG_DEBUG,
        };
        write!(&mut writer, "<{level}>[bnw@55029")?;

        let mut all_fields = std::collections::BTreeMap::new();

        if let Some(scope) = ctx.event_scope() {
            let mut last_span = "".to_string();

            for span in scope.from_root() {
                last_span = span.name().to_string();

                let ext = span.extensions();
                let values = ext.get::<FormattedValues>().expect("will never be `None`");
                for (field, value) in &values.fields {
                    all_fields.insert(field.name(), value.clone());
                }
            }

            if !all_fields.contains_key("activity") {
                all_fields.insert("activity", last_span);
            }
        }

        let mut event_fields = std::collections::HashMap::new();
        event.record(&mut CollectingVisitor::new(&mut event_fields));

        // selectively move values to `all_values`
        let mut message = None;
        for (field, value) in event_fields.drain() {
            if field.name() == "message" {
                // we want to print this outside of the field list, save it
                // for later
                message = Some(value);
            } else {
                all_fields.insert(field.name(), value);
            }
        }

        // write structured data
        // device comes first because it's value-length is constant
        let allowed_fields = ["device", "activity", "remote"];
        for (name, value) in allowed_fields
            .iter()
            .filter_map(|name| all_fields.get_key_value(name))
        {
            write!(
                writer,
                " {}=\"{}\"",
                &remove_invalid(name),
                &escape_val(value)
            )?;
        }

        // finish structured data
        write!(writer, "]")?;

        // write metrics information if any
        if all_fields.contains_key("metric_name") && all_fields.contains_key("metric_value") {
            let name = remove_invalid(&all_fields.remove("metric_name").expect("is contained"));
            let value = escape_val(&all_fields.remove("metric_value").expect("is contained"));
            write!(
                &mut writer,
                "[metric@55029 name=\"{name}\" value=\"{value}\"]"
            )?;
        }

        // write message
        write!(writer, " {}", message.as_deref().unwrap_or(""))?;

        // extend message with disallowed fields
        for (name, value) in all_fields
            .iter()
            .filter(|(name, _)| !allowed_fields.contains(name) && !name.starts_with("log."))
        {
            write!(writer, ", {name}={value}")?;
        }

        writeln!(writer)
    }
}

/// Install this tracing subscriber globally.
///
/// Works the same as [tracing_subscriber::fmt::init].
pub fn init_tracing() {
    use tracing_subscriber::field::MakeExt as _;
    use tracing_subscriber::util::SubscriberInitExt as _;

    let builder = tracing_subscriber::fmt::Subscriber::builder();
    let builder = builder.with_env_filter(tracing_subscriber::EnvFilter::from_default_env());

    if std::env::var("JOURNAL_STREAM").is_ok() {
        let formatter = tracing_subscriber::fmt::format::debug_fn(|writer, field, value| {
            let svalue = format!("{value:?}");
            write!(
                writer,
                "{}=\"{}\"",
                &remove_invalid(field.name()),
                &escape_val(&svalue)
            )
        })
        .delimited(" ");

        builder
            .event_format(MyFormatter)
            .fmt_fields(formatter)
            .finish()
            .with(ValuesLayer)
            .try_init()
            .unwrap();
    } else {
        builder.finish().try_init().unwrap();
    }
}
