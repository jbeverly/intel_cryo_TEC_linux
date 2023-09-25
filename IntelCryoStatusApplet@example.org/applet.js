const Applet = imports.ui.applet;
const GLib = imports.gi.GLib;
const St = imports.gi.St;
const PopupMenu = imports.ui.popupMenu;
const Lang = imports.lang;
const Mainloop = imports.mainloop;

function MyApplet(metadata, orientation, panelHeight, instanceId) {
  this._init(metadata, orientation, panelHeight, instanceId);
}

MyApplet.prototype = {
  __proto__: Applet.IconApplet.prototype,

  _init: function(metadata, orientation, panelHeight, instanceId) {
    this.metadata = metadata;
    Applet.IconApplet.prototype._init.call(this, orientation, panelHeight, instanceId);
    this.set_applet_icon_path(metadata.path + "/blue.png"); // Default to blue.png as the app icon
    this.set_applet_tooltip("Loading...");

    this._refresh();
    this._timeout = Mainloop.timeout_add_seconds(1, Lang.bind(this, this._refresh));
  },

  _refresh: function() {
    try {
      let fileContents = GLib.file_get_contents("/var/run/intel_cryo_tec/status.json");
      let data = JSON.parse(fileContents[1]);

      let isPidRunning = data.heartbeat["PID is running"];

      if (isPidRunning) {
        this.set_applet_icon_path(this.metadata.path + "/blue.png");
      } else {
        this.set_applet_icon_path(this.metadata.path + "/red.png");
      }

      let wattage = data.voltage * data.current;
      this.set_applet_tooltip(`Dewpoint: ${data.dewpoint.toFixed(2)}\nTemperature: ${data.temperature.toFixed(2)}\nWattage: ${wattage.toFixed(2)}`);
    } catch (error) {
      // If there's any error reading or parsing the file, set the icon to red
      this.set_applet_icon_path(this.metadata.path + "/red.png");
      this.set_applet_tooltip("Error reading the Intel Cryo TEC status file.");
    }

    return true; // This ensures the timer continues running
  },

  on_applet_clicked: function(event) {
    this._refresh();
  },

  on_applet_removed_from_panel: function() {
    // This is important to ensure we don't end up with a memory leak
    if (this._timeout) {
      Mainloop.source_remove(this._timeout);
    }
  }
};

function main(metadata, orientation, panelHeight, instanceId) {
  let myApplet = new MyApplet(metadata, orientation, panelHeight, instanceId);
  return myApplet;
}
