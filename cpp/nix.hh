#pragma once

#include <nix/primops.hh>
#include "rust/cxx.h"

using namespace nix;

struct Derivation;

rust::String addToStore(const rust::Str name, const rust::Str content);

// Wrapper around the `derivationStrict` nix builtin.
//
// The arguments are passed as json for simplicity (although that’s obviously
// not efficient nor pretty)
::Derivation derivation(const rust::Str jsonArgs);
