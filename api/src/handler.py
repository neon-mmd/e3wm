from os import path
from sys import path as spath

if path.exists(path.expanduser("~/.config/e3wm/")) and path.exists(
    path.expanduser("~/.config/e3wm/config.py")
):
    spath.insert(0, path.expanduser("~/.config/e3wm/"))
else:
    spath.insert(0, path.expanduser("/etc/xdg/e3wm"))

import config

from e3wm_api import hacker


class parse(hacker):
    def get_workspaces(self):
        return config.workspaces

    def get_layouts(self):
        return config.layouts

    def get_dynamic(self):
        return config.dynamic

    def get_keybindings(self, iterator):
        try:
            return self.bindings[iterator]
        except:
            return None


if __name__ == "__main__":
    parser = parse()
    print(parser.get_keybindings(0))
