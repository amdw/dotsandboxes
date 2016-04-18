figure_scripts := $(wildcard fig_*.py)
generated_figpdfs := $(patsubst %.py,%.pdf,$(figure_scripts))

dotsandboxes.pdf: dotsandboxes.tex $(generated_figpdfs)
	# Invoke twice to fix cross-references
	pdflatex dotsandboxes.tex
	pdflatex dotsandboxes.tex

fig_%.pdf: fig_%.svg
	inkscape -D -z --file=$< --export-pdf=$@ --export-latex

fig_%.svg: fig_%.py svg.py
	python3 $< > $@

.PHONY: clean pylint

pylint:
	python3-pylint *.py

clean:
	-rm -v *.pdf* *.svg *.pyc
