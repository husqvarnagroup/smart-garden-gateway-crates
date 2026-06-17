<!--
SPDX-FileCopyrightText: GARDENA GmbH

SPDX-License-Identifier: MIT
-->

# Public Lemonbeat XML schemas

Those files are based on the XML schemas posted publicly by Lemonbeat GmbH on
the [W3C mailing list]:

[W3C mailing list]: https://lists.w3.org/Archives/Public/public-wot-ig/2015Sep/0048.html

Steps to reproduce those files:

1. Copy all schema definitions from PDF to individual .xsd files

2. Use AI to clean up the superflous spaces. Prompt:
   > Make all files in this directory valid xsd files by removing superfluous
   > spaces.

3. Format properly:

   ```bash
   find . -name '*.xsd' -exec xmllint --output '{}' --format '{}' \;
   ```

4. Add attributes expected by our code using AI:
   > All .xsd files in this directory contain an xml schema. All schemas
   > lack "targetNamespace" and "xmlns" attributes. Add them, and use the
   > file name as value in each file, but remove the dot in the file name
   > and prefix it with "urn:".

5. Manually complement files with information stated in the PDF, but missing in
   the listed XSDs. Those missing lines seem to be cause by the XSD listings
   being split across multiple pages.
