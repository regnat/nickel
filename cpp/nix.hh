#pragma once

#include <nix/primops.hh>
#include "rust/cxx.h"

using namespace nix;

rust::String addToStore(const rust::Str name, const rust::Str content);

/* using PrimopInfo = RegisterPrimOp::Info; */

/* const std::vector<PrimopInfo> & getRegisteredPrimops(); */

/* const std::string primopName(const PrimopInfo &); */
/* const int primopArity(const PrimopInfo &); */
/* const  primop(const PrimopInfo &); */
