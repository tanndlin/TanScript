set shell := ["cmd.exe", "/c"]

asm:
    nasm -g -f win64 script.asm -o script.o
    gcc script.o -o script
    powershell -NoProfile -Command Write-Host ('Runtime: {0:N3} ms' -f (Measure-Command { .\script.exe ^| Out-Host }).TotalMilliseconds)
    del script.exe
    del script.o

profile:
    nasm -g -f win64 script.asm -o script.o
    clang --target=x86_64-w64-mingw32 --sysroot=C:/mingw64 -fuse-ld=lld script.o -o script.exe -Wl,--pdb=script.pdb
    del script.o

watch:
    nodemon -w script.asm -e asm -x "just asm"

coverage:
    cargo tarpaulin --out Html

asan_path := 'C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Tools\MSVC\14.44.35207\bin\Hostx64\x64'

fuzz *args:
    set PATH={{asan_path}};%PATH%&& cargo fuzz run compile fuzz/corpus/compile tests/golden -- {{args}}

tmin artifact:
    set PATH={{asan_path}};%PATH%&& cargo fuzz tmin compile {{artifact}}
