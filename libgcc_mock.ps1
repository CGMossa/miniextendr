# create a directory in an arbitrary location (e.g. libgcc_mock)
New-Item -Path libgcc_mock -Type Directory

# create empty libgcc_eh.a and libgcc_s.a
New-Item -Path libgcc_mock\libgcc_eh.a -Type File
New-Item -Path libgcc_mock\libgcc_s.a -Type File
$libgcc_full_path = Convert-Path .\libgcc_mock
# Set user-wide the path
[System.Environment]::SetEnvironmentVariable("LIBRARY_PATH", $libgcc_full_path, "User")

