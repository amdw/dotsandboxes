# Copyright 2016 Andrew Medworth (github@medworth.org.uk)
#
# This program is free software: you can redistribute it and/or modify
# it under the terms of the GNU Affero General Public License as published by
# the Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.
#
# This program is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU Affero General Public License for more details.
#
# You should have received a copy of the GNU Affero General Public License
# along with this program.  If not, see <http://www.gnu.org/licenses/>.

figure_scripts := $(wildcard fig_*.py)
generated_figpdfs := $(patsubst %.py,%.pdf,$(figure_scripts))

dotsandboxes.pdf: dotsandboxes.tex $(generated_figpdfs) vc.tex
	./genpdf.py

fig_%.pdf: fig_%.svg
	inkscape -D -z --file=$< --export-pdf=$@ --export-latex

fig_%.svg: fig_%.py svg.py
	python3 $< > $@

.PHONY: vc.tex clean pylint

vc.tex:
	bash vc -m

pylint:
	pylint-3 *.py

clean:
	-rm -v *.pdf* *.svg *.pyc vc.tex *.log *.aux *.toc
	-rm -rf __pycache__
