Output debug-lldb.gif

Require cargo
Require lldb
Require rust-lldb

Set Shell "bash"
Set Theme "Catppuccin Mocha"
Set FontSize 24
Set Width 1600
Set Height 1120
Set Framerate 30
Set PlaybackSpeed 1
Set TypingSpeed 25ms
Set Padding 16

Hide
Type "unset NO_COLOR"
Enter
Type "export TERM=xterm-256color COLORTERM=truecolor"
Enter
Type "cargo test --lib --no-run"
Enter
Wait+Screen@60s /Executable unittests/
Type "clear"
Enter

Show
Type "cargo test --lib tests::keeps_farthest_collinear_point -- --exact --nocapture"
Enter
Wait+Screen@30s /test result: FAILED/
Sleep 2s

Type "clear"
Enter
Sleep 500ms

Type "rust-lldb -Q target/debug/deps/graham-5c2f05a5a9fb5bb0"
Enter
Wait+Line@10s /\(lldb\)/
Sleep 1s

Type "breakpoint set --file lib.rs --line 204"
Enter
Wait+Line@10s /\(lldb\)/
Sleep 1s

Type "run tests::keeps_farthest_collinear_point --exact --nocapture"
Enter
Wait+Screen@10s /stop reason = breakpoint 1.1/
Sleep 2s

Type "frame variable pivot a b"
Enter
Wait+Line@10s /\(lldb\)/
Sleep 2s

Type "expression -- (a.x - pivot.x) * (a.x - pivot.x)"
Enter
Wait+Line@10s /\(lldb\)/
Sleep 1s

Type "expression -- (b.x - pivot.x) * (b.x - pivot.x)"
Enter
Wait+Line@10s /\(lldb\)/
Sleep 1s

Type "expression -- /* BUG FOUND: b.cmp(a) reverses the distance order */ 4 < 36"
Enter
Wait+Line@10s /\(lldb\)/
Sleep 5s
