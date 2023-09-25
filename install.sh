#!/bin/bash
cp -a ./IntelCryoStatusApplet@example.org ~/.local/share/cinnamon/applets/
sudo install -m u+x tec.py /root/tec.py
sudo cp ./intel-cryo-tec.service /etc/systemd/system/intel-cryo-tec.service
sudo systemctl daemon-reload
sudo systemctl start intel-cryo-tec.service
