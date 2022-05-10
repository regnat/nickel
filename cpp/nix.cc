#include "nix.hh"

using namespace nix;

#include <nix/config.h>
#include <nix/store-api.hh>
#include <nix/eval.hh>

#include "nickel-lang/src/eval/operation.rs.h"

static std::shared_ptr<Store> evalStore;
static std::shared_ptr<EvalState> evalState;

static std::shared_ptr<Store> getStore()
{
  if (!evalStore)
    evalStore = openStore();
  return evalStore;
}

static std::shared_ptr<EvalState> getEvalState()
{
  if (!evalState) {
    initGC();
    auto store = getStore();
    evalState = std::make_shared<EvalState>(Strings{}, ref<Store>(store));
  }
  return evalState;
}

rust::String addToStore(const rust::Str name, const rust::Str content)
{
  auto & store = *getStore(); // FIXME: ugly
  auto newPath = store.addTextToStore(std::string(name), std::string(content), {});
  return store.printStorePath(newPath);
}

::Derivation derivation(const rust::Str jsonArgs)
{
  auto & state = *getEvalState();
  auto vRes = state.allocValue();
  auto vJsonArgs = state.allocValue();
  vJsonArgs->mkString(std::string(jsonArgs));
  PathSet context;
  static auto vFun = [&](){
    auto val = state.allocValue();
    state.eval(
        state.parseExprFromString(
          R"(
            jsonArgs: (builtins.derivationStrict (builtins.fromJSON jsonArgs))
          )",
          "/"),
        *val
        );
    return val;
  }();

  state.callFunction(*vFun, *vJsonArgs, *vRes, noPos);
  /* auto res = state.coerceToPath(noPos, *vRes, context); */

  return ::Derivation {
    .drvPath = state.coerceToPath(noPos, *vRes->attrs->get(state.sDrvPath)->value, context),
    .outPath = state.coerceToPath(noPos, *vRes->attrs->get(state.symbols.create("out"))->value, context),
  };

}
