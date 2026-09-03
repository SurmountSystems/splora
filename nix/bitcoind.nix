# SPDX-License-Identifier: Unlicense
#
# Bitcoin Core 31.1 bitcoind for the splora appliance.
# Source is the bitcoincore.org tarball. This tree does not vendor C.
#
# CMake flags follow nixpkgs bitcoin 31 (BUILD_GUI, ENABLE_WALLET,
# BUILD_BENCH, WITH_ZMQ, BUILD_TESTS, BUILD_FUZZ_BINARY, BUILD_GUI_TESTS,
# WITH_BDB). Core 31.1 CMake has no WITH_SYSTEM_SECP256K1 and no
# WITH_SYSTEM_LEVELDB (unused cache variables). Gentoo's option is
# WITH_SYSTEM_LIBSECP256K1 (net-p2p/bitcoin-core
# 29.0-cmake-syslibs.patch). The patch in
# nix/patches/bitcoin-31.1-with-system-libsecp256k1.patch teaches that
# option. pkgs.secp256k1 stays in buildInputs. There is no honest
# system-leveldb switch; cmake/leveldb.cmake still vendors the tarball
# subtree. Wallet is off, so no BDB/SQLite.
{
  lib,
  stdenv,
  fetchurl,
  cmake,
  pkg-config,
  python3,
  installShellFiles,
  boost,
  libevent,
  zeromq,
  zlib,
  libsodium,
  secp256k1,
}:

stdenv.mkDerivation (finalAttrs: {
  pname = "bitcoind";
  version = "31.1";

  src = fetchurl {
    url = "https://bitcoincore.org/bin/bitcoin-core-${finalAttrs.version}/bitcoin-${finalAttrs.version}.tar.gz";
    sha256 = "50411d5b43c7e4c90099394759eb6c2add6e7c2dbe728840893d638b6fc6afc9";
  };

  nativeBuildInputs = [
    cmake
    pkg-config
    python3
    installShellFiles
  ];

  buildInputs = [
    boost
    libevent
    zeromq
    zlib
    libsodium
    secp256k1
  ];

  patches = [ ./patches/bitcoin-31.1-with-system-libsecp256k1.patch ];

  cmakeFlags = [
    (lib.cmakeBool "BUILD_GUI" false)
    (lib.cmakeBool "ENABLE_WALLET" false)
    (lib.cmakeBool "WITH_BDB" false)
    (lib.cmakeBool "BUILD_BENCH" false)
    (lib.cmakeBool "WITH_ZMQ" true)
    (lib.cmakeBool "BUILD_TESTS" false)
    (lib.cmakeBool "BUILD_FUZZ_BINARY" false)
    (lib.cmakeBool "BUILD_GUI_TESTS" false)
    (lib.cmakeBool "ENABLE_IPC" false)
    (lib.cmakeBool "WITH_SYSTEM_LIBSECP256K1" true)
  ];

  doCheck = false;

  meta = {
    description = "Bitcoin Core 31.1 daemon (wallet off, no GUI) for the splora appliance";
    homepage = "https://bitcoincore.org/";
    license = lib.licenses.mit;
    platforms = lib.platforms.linux;
    mainProgram = "bitcoind";
  };
})
