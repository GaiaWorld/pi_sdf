@echo off
setlocal
:: 设置默认路径
set "projectRoot="

if exist ../temp/cfg.txt (for /f "delims=" %%i in (../temp/cfg.txt) do set "%%i")

:: 让用户输入路径
set /p "projectRoot1=projectRoot path (%projectRoot%): "

:: 如果用户没有输入任何内容，那么就使用默认路径
if "%projectRoot1%"=="" (
    set "projectRoot1=%projectRoot%"
)

:: 如果不存在temp目录，则创建
set directory=../temp
if not exist "%directory%" (
    mkdir "%directory%"
)

:: 将新的配置写入配置文件中
(echo projectRoot=%projectRoot1%) >../temp/cfg.txt
endlocal