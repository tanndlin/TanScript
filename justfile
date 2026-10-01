set shell := ["cmd.exe", "/c"]

asm:
    nasm -g -f win64 script.asm -o script.o
    gcc script.o -o script
    powershell -NoProfile -Command Write-Host ('Runtime: {0:N3} ms' -f (Measure-Command { .\script.exe ^| Out-Host }).TotalMilliseconds)
    del script.exe
    del script.o

watch:
    nodemon -e asm -x "just asm"

coverage:
    cargo tarpaulin --out Html