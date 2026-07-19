if [[ $EUID -ne 0 ]]; then
   echo "Please run as root" 
   exit 1
fi

if ! pkg-config --exists netcdf; then
   echo "WARNING netcdf NOT FOUND"
   echo "Install first with"
   echo "   Arch: sudo pacman -S netcdf"
fi

workdir=$(pwd)

target_bin_dir="/usr/bin"

latest_web_url="https://github.com/LorgaschLunka/hostrada-netcsv/releases/latest/download/hostrada-netcsv"

echo "Downloading hostrada-netcsv.bin to $target_bin_dir"
sudo curl -sSL -o "$target_bin_dir/hostrada-netcsv" $latest_web_url

echo "Adding execute permissions for binary"
sudo chmod +x $target_bin_dir/hostrada-netcsv

echo "Done"

