$ErrorActionPreference = "Stop"
Write-Host "Build Qt desktop (requires Qt 6 + CMake + MSVC)"
cmake -S ..\desktop -B ..\desktop\build -DCMAKE_BUILD_TYPE=Release
cmake --build ..\desktop\build --config Release
Write-Host "Build Java backend"
Push-Location ..\backend
mvn -DskipTests package
Pop-Location
Write-Host "Build complete. Package Qt dependencies with windeployqt and include backend JAR + JRE."
