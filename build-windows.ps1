$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $MyInvocation.MyCommand.Path
$Build = Join-Path $Root "build"
Write-Host "Building Doctor AI desktop application..."
cmake -S $Root -B $Build -DCMAKE_BUILD_TYPE=Release
cmake --build $Build --config Release
Write-Host "Building Doctor AI Java backend..."
mvn -f (Join-Path $Root "pom.xml") -DskipTests package
Write-Host "Build completed. Desktop executable and backend JAR are in the build output folders."
