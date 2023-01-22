# update_chrome_driver
A command that downloads the correct version of chromedriver regarding the version of the local Chromium installed.  

## Usage
```
$> update_chrome_driver.exe <CHROME_BROWSER_PATH> <OUTPUT_DIRECTORY>

Arguments:
  <CHROME_BROWSER_PATH>  The location of the local Google Chrome executable
  <OUTPUT_DIRECTORY>     The location of the output directory where the Google Driver executable will be extracted

Options:
  -h, --help  Print help

```
## Windows
Due to this [bug](https://bugs.chromium.org/p/chromium/issues/detail?id=158372) in Chromium, [WMIC](https://learn.microsoft.com/en-us/windows/win32/wmisdk/wmic) is needed on Windows.

