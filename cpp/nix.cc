#include "nix.hh"

using namespace nix;

#include <nix/config.h>
#include <nix/store-api.hh>

rust::String addToStore(const rust::Str name, const rust::Str content)
{
  auto store = openStore(); // FIXME: ugly
  auto newPath = store->addTextToStore(std::string(name), std::string(content), {});
  return store->printStorePath(newPath);
}

/* const std::vector<PrimopInfo> & getRegisteredPrimops() */
/* { */
/*   return *RegisterPrimOp::primOps; */
/* } */
