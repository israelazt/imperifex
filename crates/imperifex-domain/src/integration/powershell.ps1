# Imperifex: shell integration. It runs after your profile, which it does not
# edit: it keeps your prompt and says where it starts and ends (OSC 133).
$global:__ImperifexPrompt = $function:prompt
$global:__ImperifexRan = $false
function global:prompt {
    $ok = $global:?
    $code = $global:LASTEXITCODE
    $e = [char]27
    $b = [char]7
    $marks = ''
    if ($global:__ImperifexRan) {
        $result = 0
        if (-not $ok) { $result = 1; if ($code) { $result = $code } }
        $marks = "$e]133;D;$result$b"
        $global:__ImperifexRan = $false
    }
    $global:LASTEXITCODE = $code
    $text = & $global:__ImperifexPrompt
    "$marks$e]133;A$b$text$e]133;B$b"
}
if (Test-Path function:PSConsoleHostReadLine) {
    $global:__ImperifexReadLine = $function:PSConsoleHostReadLine
    function global:PSConsoleHostReadLine {
        $line = & $global:__ImperifexReadLine
        if ($line.Trim()) {
            $global:__ImperifexRan = $true
            [Console]::Write("$([char]27)]133;C$([char]7)")
        }
        $line
    }
}
