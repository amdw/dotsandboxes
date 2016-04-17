dotsandboxes.pdf: dotsandboxes.tex fig_p45.pdf
	# Invoke twice to fix cross-references
	pdflatex dotsandboxes.tex
	pdflatex dotsandboxes.tex

fig_p45.pdf: fig_p45.svg
	inkscape -D -z --file=fig_p45.svg --export-pdf=fig_p45.pdf --export-latex

fig_p45.svg: fig_p45.py svg.py
	python3 fig_p45.py > fig_p45.svg

.PHONY: clean
clean:
	-rm -v *.pdf* *.svg *.pyc
