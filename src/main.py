# main.py
#
# Copyright 2023 Iman Salmani
#
# This program is free software: you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.
#
# This program is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU General Public License for more details.
#
# You should have received a copy of the GNU General Public License
# along with this program.  If not, see <http://www.gnu.org/licenses/>.
#
# SPDX-License-Identifier: GPL-3.0-or-later

import sys
import gi

gi.require_version('Gtk', '4.0')
gi.require_version('Adw', '1')

from gi.repository import Gtk, Gio, Adw
from iplan.views.window import IplanWindow


class IplanApplication(Adw.Application):
    """The main application singleton class."""

    def __init__(self):
        super().__init__(application_id='ir.imansalmani.iplan',
                         flags=Gio.ApplicationFlags.FLAGS_NONE)
        self.create_action('quit', lambda *args: self.quit(), ['<Ctrl>q'])
        self.create_action('about', self.on_about)
        self.create_action('shortcuts', self.on_shortcuts, ['<Ctrl>question'])
        self.create_action('preferences', self.on_preferences, ['<Ctrl>comma'])

    def do_activate(self):
        """Called when the application is activated.

        We raise the application's main window, creating it if
        necessary.
        """
        win = self.props.active_window
        if not win:
            win = IplanWindow(application=self)
        win.present()

    def on_about(self, widget, _):
        """Callback for the app.about action."""
        about = Adw.AboutWindow(transient_for=self.props.active_window,
                                application_name='iplan',
                                application_icon='ir.imansalmani.iplan',
                                developer_name='Iman Salmani',
                                version='0.1.0',
                                developers=['Iman Salmani'],
                                copyright='© 2023 Iman Salmani')
        about.present()

    def on_shortcuts(self, widget, _):
        shortcuts_window = ShortcutsWindow()
        shortcuts_window.set_transient_for(self.props.active_window)
        shortcuts_window.present()

    def on_preferences(self, widget, _):
        """Callback for the app.preferences action."""
        print('app.preferences action activated')

    def create_action(self, name, callback, shortcuts=None):
        """Add an application action.

        Args:
            name: the name of the action
            callback: the function to be called when the action is
              activated
            shortcuts: an optional list of accelerators
        """
        action = Gio.SimpleAction.new(name, None)
        action.connect("activate", callback)
        self.add_action(action)
        if shortcuts:
            self.set_accels_for_action(f"app.{name}", shortcuts)


@Gtk.Template(resource_path="/ir/imansalmani/iplan/ui/shortcuts_window.ui")
class ShortcutsWindow(Gtk.ShortcutsWindow):
    __gtype_name__ = "ShortcutsWindow"

def main(version):
    """The application's entry point."""
    app = IplanApplication()
    return app.run(sys.argv)
