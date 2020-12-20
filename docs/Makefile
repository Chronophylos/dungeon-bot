ASCIIDOCTOR=asciidoctor
ADOCFLAGS=-vtwb xhtml5 -r asciidoctor-diagram

build: *.adoc
	$(ASCIIDOCTOR) $(ADOCFLAGS) $?

clean:
	-rm *.svg
	-rm *.html
	-rm -rf .asciidoctor

.PHONY: clean
