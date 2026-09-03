# SPDX-License-Identifier: Unlicense
#
# Elements 23.3.3 elementsd for Liquid on the splora appliance.
# Tag elements-23.3.3 is autotools (configure.ac). It is not CMake.
# Wallet off, no GUI, no BDB. secp256k1 and leveldb stay in-tree in this
# tag; configure.ac on elements-23.3.3 does not expose a WITH_SYSTEM
# switch. pkgs.secp256k1, pkgs.leveldb, boost, and libevent are still
# linked where the build allows (boost and libevent are real
# --with-boost-libdir / pkg-config deps).
#
# sha256 is the unpacked GitHub archive for tag elements-23.3.3
# (nix-prefetch-url --unpack, 2026-09-02).
{
  lib,
  stdenv,
  fetchFromGitHub,
  autoreconfHook,
  pkg-config,
  util-linux,
  python3,
  hexdump,
  boost,
  libevent,
  zeromq,
  zlib,
  secp256k1,
  leveldb,
}:

stdenv.mkDerivation rec {
  pname = "elementsd";
  version = "23.3.3";

  src = fetchFromGitHub {
    owner = "ElementsProject";
    repo = "elements";
    rev = "elements-${version}";
    sha256 = "07p0zknrz74jyvxm04pa20y35kdarp9y0f5k99xz72psx9achkxv";
  };

  nativeBuildInputs = [
    autoreconfHook
    pkg-config
    python3
    hexdump
  ]
  ++ lib.optionals stdenv.hostPlatform.isLinux [ util-linux ];

  buildInputs = [
    boost
    libevent
    zeromq
    zlib
    secp256k1
    leveldb
  ];

  configureFlags = [
    "--with-boost-libdir=${lib.getLib boost}/lib"
    "--disable-wallet"
    "--without-gui"
    "--without-miniupnpc"
    "--without-natpmp"
    "--disable-bench"
    "--disable-tests"
    "--disable-gui-tests"
    "--disable-fuzz"
    "--disable-fuzz-binary"
    "--with-daemon"
  ];

  doCheck = false;

  meta = {
    description = "Elements 23.3.3 daemon (wallet off, no GUI) for Liquid on the splora appliance";
    homepage = "https://github.com/ElementsProject/elements";
    license = lib.licenses.mit;
    platforms = lib.platforms.linux;
    mainProgram = "elementsd";
  };
}
