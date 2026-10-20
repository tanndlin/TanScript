set shell := ["cmd.exe", "/c"]

asm file="script.asm":
    nasm -g -f win64 {{file}} -o {{replace(without_extension(file), '/', '')}}.o
    gcc {{replace(without_extension(file), '/', '')}}.o -o {{replace(without_extension(file), '/', '')}}
    powershell -NoProfile -Command Write-Host ('Runtime: {0:N3} ms' -f (Measure-Command { .\{{replace(without_extension(file), '/', '')}}.exe ^| Out-Host }).TotalMilliseconds)
    del {{replace(without_extension(file), '/', '')}}.exe
    del {{replace(without_extension(file), '/', '')}}.o

profile:
    nasm -g -f win64 script.asm -o script.o
    clang --target=x86_64-w64-mingw32 --sysroot=C:/mingw64 -fuse-ld=lld script.o -o script.exe -Wl,--pdb=script.pdb
    del script.o

watch file="script.asm":
    nodemon -w {{file}} -e asm -x "just asm {{file}}"

coverage:
    cargo tarpaulin --out Html

asan_path := 'C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Tools\MSVC\14.44.35207\bin\Hostx64\x64'

fuzz *args:
    set PATH={{asan_path}};%PATH%&& cargo fuzz run compile fuzz/corpus/compile tests/golden -- -dict=fuzz/compile.dict -max_len=1024 {{args}}

tmin artifact:
    set PATH={{asan_path}};%PATH%&& cargo fuzz tmin compile {{artifact}}

install-extension:
    -del %USERPROFILE%\.cargo\bin\tanscript-lsp.old.exe 2>nul
    -move /y %USERPROFILE%\.cargo\bin\tanscript-lsp.exe %USERPROFILE%\.cargo\bin\tanscript-lsp.old.exe >nul 2>nul
    cargo install --path lsp
    -taskkill /f /im tanscript-lsp.exe >nul 2>nul
    cd editors\vscode && npx --yes @vscode/vsce package --allow-missing-repository --skip-license -o tanscript.vsix && code --install-extension tanscript.vsix --force
