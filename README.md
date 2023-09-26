# intel_cryo_TEC_linux
A quick'n'dirty (And I do mean dirty...) way to use the EKWB Intel Cryo cooler on a linux desktop. 

# Warning:
- I have only tried this on my Intel 10900K. The PID settings I'm using may not work for you. They are pretty generic, but YMMV.

# Notes:
- Because, for some reason, I chose to write this in python, it's 6.1M in RSS... I feel bad.**UPDATE** I wrote the daemon again in  RUST, and that's just over 1k RSS.  
- The applet piece works in Mint Cinnamon because that's the desktop I chose to use on the PC that has the EKWB QuantumX Tec Delta. Feel free to contribute applets for your WM of choice (or I'll do it if I move to another WM)
- I didn't bother implementing unregulated mode because I don't use it. 

Image attribution:
- blue.png: Image by [Vectorportal.com](https://www.vectorportal.com), [CC BY](https://creativecommons.org/licenses/by/4.0/) (modified)
- red.png: Image by [Vectorportal.com](https://www.vectorportal.com), [CC BY](https://creativecommons.org/licenses/by/4.0/) (modified)

# Installation:
Read, and if you like it, run the [install.sh](install.sh) 

