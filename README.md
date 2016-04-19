# Dots-and-boxes

This repository is dedicated to the dots-and-boxes game.

So far, the only thing here is a draft of a paper I am writing on the game.

This needs a full installation of [LaTeX](http://latex-project.org/),
as well as Python 3 and Inkscape to generate some of the diagrams, and
`make` to coordinate the build. E.g. on Fedora, `dnf install texlive
texlive-svg inkscape pstoedit python3 make`, `cd` into this repository, and run
`make`.

To perform [Pylint](https://www.pylint.org/) checks on the Python
scripts, run `make pylint` (requires `python3-pylint`). It
produces quite a bit of output so you may want to pipe it through
`less`.

I would be very grateful to learn of any errors in this material.

# Licensing

All material in this repository is Copyright 2016 Andrew Medworth
(github@medworth.org.uk).

The Dots-and-Boxes paper is licensed under a Creative Commons license
as follows.

<a rel="license" href="http://creativecommons.org/licenses/by-nc-sa/4.0/"><img alt="Creative Commons License" style="border-width:0" src="https://i.creativecommons.org/l/by-nc-sa/4.0/88x31.png" /></a><br /><span xmlns:dct="http://purl.org/dc/terms/" property="dct:title">Dots-and-Boxes</span> by <a xmlns:cc="http://creativecommons.org/ns#" href="https://github.com/amdw/dotsandboxes" property="cc:attributionName" rel="cc:attributionURL">Andrew Medworth</a> is licensed under a <a rel="license" href="http://creativecommons.org/licenses/by-nc-sa/4.0/">Creative Commons Attribution-NonCommercial-ShareAlike 4.0 International License</a>.<br />Based on a work at <a xmlns:dct="http://purl.org/dc/terms/" href="https://github.com/amdw/dotsandboxes" rel="dct:source">https://github.com/amdw/dotsandboxes</a>.

The software in this repository is licensed under the GNU Affero
General Public License, as per the file LICENSE.txt.

I am happy to consider requests to re-license this software under a
non-copyleft open source license. Just get in touch with me to let me
know what you want to do and how the current license is
inconveniencing you.