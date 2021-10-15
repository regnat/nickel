use std::{collections::HashMap, marker::PhantomData, str::EncodeUtf16};

use super::reporting::NameResolution;
use super::{State, TypeWrapper, UnifTable};
use crate::environment::Environment as GenericEnvironment;
use crate::typecheck::to_type;
use crate::types::{AbsType, Types};
use crate::{identifier::Ident, position::TermPos, term::Term};

pub struct Linearization<LinearizationState> {
    pub state: LinearizationState,
}

impl Linearization<()> {
    pub fn completed(completed: Completed) -> Linearization<Completed> {
        Linearization { state: completed }
    }
    pub fn building<T: Default>() -> Linearization<Building<T>> {
        Linearization {
            state: Building {
                resource: T::default(),
            },
        }
    }
}

pub struct Building<T> {
    resource: T,
}

#[derive(Debug)]
pub struct Completed {
    pub lin: Vec<LinearizationItem<Resolved>>,
    pub id_mapping: HashMap<usize, usize>,
    pub scope_mapping: HashMap<Vec<ScopeId>, Vec<usize>>,
}

pub trait ResolutionState {}
type Resolved = Types;
impl ResolutionState for Resolved {}

type Unresolved = TypeWrapper;
impl ResolutionState for Unresolved {}

trait LinearizationState {}
impl<T> LinearizationState for Building<T> {}

impl LinearizationState for () {}

#[derive(Debug, Clone, PartialEq)]
pub struct LinearizationItem<ResolutionState> {
    //term_: Box<Term>,
    pub id: usize,
    pub pos: TermPos,
    pub ty: ResolutionState,
    pub kind: TermKind,
    scope: Vec<ScopeId>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TermKind {
    Structure,
    Declaration(String, Vec<usize>),
    Usage(Option<usize>),
}

pub trait Linearizer<L, S> {
    fn add_term(
        &mut self,
        lin: &mut Linearization<Building<L>>,
        term: &Term,
        pos: TermPos,
        ty: TypeWrapper,
    ) {
    }
    fn linearize(self, lin: Linearization<Building<L>>, extra: &S) -> Linearization<Completed>
    where
        Self: Sized,
    {
        Linearization {
            state: Completed {
                lin: Vec::new(),
                id_mapping: HashMap::new(),
                scope_mapping: HashMap::new(),
            },
        }
    }
    fn scope(&self, branch: ScopeId) -> Self;
}

pub struct StubHost<L>(PhantomData<L>);
impl<L, S> Linearizer<L, S> for StubHost<L> {
    fn scope(&self, _: ScopeId) -> Self {
        StubHost::new()
    }
}

impl<L> StubHost<L> {
    pub fn new() -> StubHost<L> {
        StubHost(PhantomData)
    }
}

pub type Environment = GenericEnvironment<Ident, usize>;
pub struct AnalysisHost {
    env: Environment,
    scope: Vec<ScopeId>,
}

impl AnalysisHost {
    pub fn new() -> Self {
        AnalysisHost {
            env: Environment::new(),
            scope: Vec::new(),
        }
    }
}

trait ScopeIdElem: Clone + Eq {}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum ScopeId {
    Left,
    Right,
    Choice(usize),
}

impl ScopeIdElem for ScopeId {}

#[derive(Default)]
pub struct BuildingResource {
    linearization: Vec<LinearizationItem<Unresolved>>,
    scope: HashMap<Vec<ScopeId>, Vec<usize>>,
}

impl Linearizer<BuildingResource, UnifTable> for AnalysisHost {
    fn add_term(
        &mut self,
        lin: &mut Linearization<Building<BuildingResource>>,
        term: &Term,
        pos: TermPos,
        ty: TypeWrapper,
    ) {
        if pos == TermPos::None {
            eprintln!("{:?}", term);
            return;
        }
        let id = lin.state.resource.linearization.len();
        match term {
            Term::Let(ident, definition, _) => {
                self.env
                    .insert(ident.to_owned(), lin.state.resource.linearization.len());
                lin.push(LinearizationItem {
                    id,
                    ty,
                    pos: definition.pos,
                    scope: self.scope.clone(),
                    kind: TermKind::Declaration(ident.to_string(), Vec::new()),
                });
            }
            Term::Var(ident) => {
                let parent = self.env.get(ident);
                lin.push(LinearizationItem {
                    id,
                    pos,
                    ty,
                    scope: self.scope.clone(),
                    kind: TermKind::Usage(parent),
                });
                if let Some(parent) = parent {
                    lin.add_usage(parent, id);
                }
            }
            _ => lin.push(LinearizationItem {
                id,
                pos,
                ty,
                scope: self.scope.clone(),
                kind: TermKind::Structure,
            }),
        }
    }

    fn linearize(
        self,
        lin: Linearization<Building<BuildingResource>>,
        extra: &UnifTable,
    ) -> Linearization<Completed> {
        let mut lin_ = lin.state.resource.linearization;
        eprintln!("linearizing");
        lin_.sort_by_key(|item| match item.pos {
            TermPos::Original(span) => (span.src_id, span.start),
            TermPos::Inherited(span) => (span.src_id, span.start),
            TermPos::None => {
                eprintln!("{:?}", item);

                unreachable!()
            }
        });

        let mut id_mapping = HashMap::new();
        lin_.iter()
            .enumerate()
            .for_each(|(index, LinearizationItem { id, .. })| {
                id_mapping.insert(*id, index);
            });

        let lin_ = lin_
            .into_iter()
            .map(
                |LinearizationItem {
                     id,
                     pos,
                     ty,
                     kind,
                     scope,
                 }| LinearizationItem {
                    ty: to_type(extra, ty),
                    id,
                    pos,
                    kind,
                    scope,
                },
            )
            .collect();

        Linearization::completed(Completed {
            lin: lin_,
            id_mapping,
            scope_mapping: lin.state.resource.scope,
        })
    }

    fn scope(&self, scope_id: ScopeId) -> Self {
        let mut scope = self.scope.clone();
        scope.push(scope_id);

        AnalysisHost {
            scope,
            env: self.env.clone(),
        }
    }
}

impl Linearization<Building<BuildingResource>> {
    fn push(&mut self, item: LinearizationItem<Unresolved>) {
        self.state
            .resource
            .scope
            .remove(&item.scope)
            .map(|mut s| {
                s.push(item.id);
                s
            })
            .or_else(|| Some(vec![item.id]))
            .into_iter()
            .for_each(|l| {
                self.state.resource.scope.insert(item.scope.clone(), l);
            });
        self.state.resource.linearization.push(item);
    }

    fn add_usage(&mut self, decl: usize, usage: usize) {
        match self
            .state
            .resource
            .linearization
            .get_mut(decl)
            .expect("Coundt find parent")
            .kind
        {
            TermKind::Structure => unreachable!(),
            TermKind::Usage(_) => unreachable!(),
            TermKind::Declaration(_, ref mut usages) => usages.push(usage),
        };
    }
}

impl Linearization<Completed> {
    pub fn get_item(&self, id: usize) -> Option<&LinearizationItem<Resolved>> {
        self.state
            .id_mapping
            .get(&id)
            .and_then(|index| self.state.lin.get(*index))
    }

    pub fn get_in_scope(
        &self,
        LinearizationItem { scope, .. }: &LinearizationItem<Resolved>,
    ) -> Vec<&LinearizationItem<Resolved>> {
        (0..scope.len())
            .into_iter()
            .map(|end| &scope[..end])
            .flat_map(|scope| {
                self.state
                    .scope_mapping
                    .get(scope)
                    .map_or_else(|| Vec::new(), Clone::clone)
            })
            .map(|id| self.get_item(id))
            .filter(Option::is_some)
            .map(Option::unwrap)
            .collect()
    }
}
