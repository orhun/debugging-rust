Output demo.gif

Require cargo

Set Shell "bash"
Set Theme "Catppuccin Mocha"
Set FontSize 26
Set Width 1600
Set Height 1120
Set Framerate 30
Set PlaybackSpeed 1
Set Padding 12

Hide
Type "env -u NO_COLOR TERM=xterm-256color COLORTERM=truecolor cargo run --quiet"
Enter
Wait+Screen@60s /Graham Scan TUI/

Show
Sleep 1s
Enter
Wait+Screen@10s /FAILED/
Sleep 5s
