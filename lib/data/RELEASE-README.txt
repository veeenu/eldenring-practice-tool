Welcome to the Elden Ring Practice Tool!

----------------------------------------------------------------------------------------------------
# How to run the tool

Always double-click `eldenring.exe` to start the game (Steam → right click **ELDEN RING** →
Manage → Browse Local Files). Never start the game from Steam: if you do, the tool won't work.

You can run the practice tool in two ways.

By double-clicking `jdsd_er_practice_tool.exe`:
  - Extract all files from the zip archive. Anywhere will do.
  - Double-click `jdsd_er_practice_tool.exe` while the game is running.

By installing it:
  - Copy all the files to your ELDEN RING game folder, the one that contains `eldenring.exe`.
    Repeat this for every patch of the game you play on.
  - Start ELDEN RING by double-clicking `eldenring.exe`.
  - While it is starting, press and hold right shift for a few seconds, until the tool appears.
    If you miss it, double-click `jdsd_er_practice_tool.exe` in the game folder.
  - To update, copy the new files over the old ones. To uninstall, delete `dinput8.dll`.

The tool always loads the `dinput8.dll` next to the `jdsd_er_practice_tool.exe` you
double-click: to try a new version, start the game without holding right shift, then run the
new version's exe.

The first time you run the tool, if you haven't disabled EAC already, the tool will offer to do it
for you. You will then have to restart the game, and then the tool, and you'll be good to go!

On Linux, set the game's launch options in Steam to `WINEDLLOVERRIDES="dinput8=n,b" %command%`.
----------------------------------------------------------------------------------------------------

If you have issues or questions, thoroughly check the FAQ.
If you can't find what you need there, feel free to open an issue or join the discord.

You can find the FAQ here:
https://github.com/veeenu/eldenring-practice-tool#readme

If you enjoy the tool, please consider supporting me:
https://patreon.com/johndisandonato

-- johndisandonato
