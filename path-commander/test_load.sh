#!/bin/bash
echo "Testing theme loading..."
echo ""
echo "Checking if dracula.ini exists:"
ls -lh ~/.pc/themes/dracula.ini
echo ""
echo "First 30 lines of dracula.ini:"
head -30 ~/.pc/themes/dracula.ini
echo ""
echo "Running pc with dracula theme (capture any errors):"
./target/release/pc.exe --theme dracula 2>&1 | head -5
