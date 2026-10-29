OSATURA Portable - how the PortableApps.com package is built
============================================================

1. Assemble the layout (any OS, Python 3):
     python3 -I packaging/portableapps/make_package.py osatura-windows-x86_64.zip 0.1.0-alpha.5 out
   -> out/OSATURAPortable/ with App\OSATURA (the release files), App\AppInfo (appinfo.ini, icons,
      Launcher\OSATURAPortable.ini), App\DefaultData and help.html.

2. On Windows, with the PortableApps.com Launcher (https://portableapps.com/apps/development/portableapps.com_launcher):
   run "PortableApps.com Launcher Generator" on the OSATURAPortable folder. It reads
   App\AppInfo\Launcher\OSATURAPortable.ini and writes OSATURAPortable.exe.

3. Run "PortableApps.com Installer" on the same folder to make OSATURAPortable_<version>.paf.exe.

4. Test: start OSATURAPortable.exe on a clean Windows account and check that nothing appears in
   %APPDATA%\OSATURA, Documents\OSATURA or the registry, and that Data\ fills with settings, Projects and WebView2.

The environment variables set by the launcher (OSATURA_UI_CONFIG, OSATURA_DOCUMENTS, WEBVIEW2_USER_DATA_FOLDER) are
documented in docs/UNINSTALL.md and docs/STUDIO.md of the source.
