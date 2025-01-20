gcc vm.c
# Make sure it ran
if ($LASTEXITCODE -ne 0) {
    Write-Output "Compilation failed"
    exit 1
}

$ms = Measure-Command { ./a.exe script.tsc > out.txt } | Select-Object -ExpandProperty TotalMilliseconds
Get-Content out.txt

Write-Output "Runtime: $ms ms"
Remove-Item out.txt
Remove-Item a.exe