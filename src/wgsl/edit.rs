use lazy_static::__Deref;

use super::{fn_decl::*, overload_row::*, ty::*};

pub struct TypeParamNotFound;

impl UnionBound {
    /// returns Err(()) if `param` is not mentioned in `f`
    pub fn instantiate_type_param(
        &self,
        param: &Ident,
        f: &FnDecl,
    ) -> Result<Vec<FnDecl>, TypeParamNotFound> {
        let t: Ty = param.clone().into();

        let mut contains_param = false;
        contains_param |= f.args.iter().any(|(_, arg)| arg.find(&t).is_some());
        contains_param |= f.out.find(&t).is_some();

        let true = contains_param else {Err(TypeParamNotFound)?};

        let instances = self.iter().map(|variant| {
            let mut f = f.clone();
            let replace = |ty: &mut Ty| *ty = variant.clone();
            f.args
                .iter_mut()
                .filter_map(|(_, ty)| ty.find_mut(&t))
                .for_each(replace);
            f.out.find_mut(&t).map(replace);
            f
        });
        Ok(instances.collect())
    }

    /// tries to apply the bound to as many elements of `fs` as possible.
    ///
    /// collects all function instantiations in a new vec
    pub fn instantiate_type_param_multi(&self, param: &Ident, fs: &[FnDecl]) -> Vec<FnDecl> {
        fs.iter()
            .flat_map(|f| match self.instantiate_type_param(param, f) {
                Ok(instances) => instances,
                Err(TypeParamNotFound) => vec![f.clone()],
            })
            .collect()
    }
}

impl OverloadRow {
    /// looks through the type parameter bounds and applies them if possible
    ///
    /// union bounds like `T is i32 or u32` can be applied by replacing every
    /// `T` argument in the function with `i32` and `u32` to yield 2 separate
    /// overloads.
    ///
    /// two bounds like this would yield 4 overloads for every permutation
    /// of union bound variants and so on...
    pub fn instantiate_bounds_if_possible(&self) -> Vec<OverloadRow> {
        // only union bounds can be used to instantiated functions so far
        // the rest of the bounds must be kept in the parametrization
        let leftover: Vec<_> = self
            .parametrization
            .iter()
            .filter(|b| !matches!(b.bound_kind, BoundKind::Union(_)))
            .cloned()
            .collect();

        self.parametrization
            .iter()
            .filter_map(|b| match &b.bound_kind {
                BoundKind::Union(union) => Some((&b.type_param, union)),
                _ => None,
            })
            // instantiate the union variants step by step
            // accumulate the new overloads
            .fold(vec![self.fn_decl.clone()], |acc, (p, bound)| {
                bound.instantiate_type_param_multi(p, &acc)
            })
            .into_iter()
            // create new overload row per instance with the leftover bounds
            .map(|fn_decl| OverloadRow {
                algorithm_attr: self.algorithm_attr.clone(),
                parametrization: Parametrization(leftover.clone()),
                fn_decl,
            })
            .collect()
    }
}
