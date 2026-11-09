# Copyright 2016-2026 Andrew Medworth (github@medworth.org.uk)
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

# All generated files go in this directory
build := build

figure_scripts := $(wildcard fig_*.py)
generated_figpdfs := $(patsubst %.py,$(build)/%.pdf,$(figure_scripts))

.PHONY: all $(build)/vc.tex clean pylint

all: $(build)/dotsandboxes.pdf

$(build)/dotsandboxes.pdf: dotsandboxes.tex $(generated_figpdfs) $(build)/vc.tex genpdf.py
	./genpdf.py $(build)

$(build)/fig_%.pdf: $(build)/fig_%.svg
	inkscape -D --export-type=pdf --export-latex --export-filename=$@ $<

$(build)/fig_%.svg: fig_%.py svg.py | $(build)
	python3 -B $< > $@

# vc always writes vc.tex in the current directory
$(build)/vc.tex: | $(build)
	bash vc -m
	mv vc.tex $@

$(build):
	mkdir -p $@

pylint:
	pylint-3 *.py

clean:
	-rm -rf $(build) __pycache__
