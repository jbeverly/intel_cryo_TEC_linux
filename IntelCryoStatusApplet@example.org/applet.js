const Applet = imports.ui.applet;
const GLib = imports.gi.GLib;
const Gio = imports.gi.Gio;
const St = imports.gi.St;
const PopupMenu = imports.ui.popupMenu;
const Lang = imports.lang;
const Mainloop = imports.mainloop;

function IntelCryoTEC(metadata, orientation, panelHeight, instanceId) {
  this._init(metadata, orientation, panelHeight, instanceId);
}

IntelCryoTEC.prototype = {
  __proto__: Applet.IconApplet.prototype,

  _init: function(metadata, orientation, panelHeight, instanceId) {
    Applet.IconApplet.prototype._init.call(this, orientation, panelHeight, instanceId);

    // Set the metadata
    this.metadata = metadata;

    // Set the default icon
    this.set_applet_icon_path(this.metadata.path + "/blue.png");
    this.set_applet_tooltip("Loading...");

    // Initialize the menu
    this.menuManager = new PopupMenu.PopupMenuManager(this);
    this.menu = new Applet.AppletPopupMenu(this, orientation);
    this.menuManager.addMenu(this.menu);

    // Start the refreshing process
    this._refresh();
    this._timeout = Mainloop.timeout_add_seconds(1, Lang.bind(this, this._refresh));
  },

  _error_state: function(message) {
    this.set_applet_icon_path(this.metadata.path + "/red.png");
    this.set_applet_tooltip(message);
  },

  _okay_state: function(message) {
    this.set_applet_icon_path(this.metadata.path + "/blue.png");
    this.set_applet_tooltip(message);
  },

  _refresh: function() {
    try {
      let fileContents = GLib.file_get_contents("/var/run/intel_cryo_tec/status.json");
      let data = JSON.parse(fileContents[1]);

      let isPidRunning = data.heartbeat["PID is running"];
      let timestamp = data.timestamp
      let current_timestamp = Math.floor(Date.now() / 1000);

      if ((current_timestamp - timestamp) > 5) {
        this._error_state("Service not running")
      } else if (!isPidRunning) {
        this._error_state("Cooler is in standby")
      } else {
        let wattage = data.voltage * data.current;
        this._okay_state(`Dewpoint: ${data.dewpoint.toFixed(2)}\nTemperature: ${data.temperature.toFixed(2)}\nWattage: ${wattage.toFixed(2)}`);
      }

      this._refreshMenu();
    } catch (error) {
      this._error_state("Error reading the Intel Cryo TEC status file.")
    }

    return true; // This ensures the timer continues running
  },

  _refreshMenu: function() {
    this.menu.removeAll(); // Clear the current menu items

    try {
      let fileContents = GLib.file_get_contents("/var/run/intel_cryo_tec/status.json");
      let data = JSON.parse(fileContents[1]);

      // Pretty print the JSON data
      let formattedData = JSON.stringify(data, null, 4);

      // Add the formatted data to the menu
      let menuItem = new PopupMenu.PopupMenuItem(formattedData, { reactive: false });
      this.menu.addMenuItem(menuItem);
    } catch (error) {
      let menuItem = new PopupMenu.PopupMenuItem("Error reading the Intel Cryo TEC status file.", { reactive: false });
      this.menu.addMenuItem(menuItem);
    }
  },

  on_applet_clicked: function(event) {
    this._refresh();
    this.menu.toggle(); // Toggle the visibility of the menu
  },

  on_applet_removed_from_panel: function() {
    // This is important to ensure we don't end up with a memory leak
    if (this._timeout) {
      Mainloop.source_remove(this._timeout);
    }
  }
};

function main(metadata, orientation, panelHeight, instanceId) {
  let intelCryoTec = new IntelCryoTEC(metadata, orientation, panelHeight, instanceId);
  return intelCryoTec;
}

