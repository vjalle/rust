use rustc_abi::{BackendRepr, FieldsShape, HasDataLayout, Primitive, Size, TyAbiInterface};

use crate::callconv::{ArgAbi, FnAbi, Reg, Uniform};
use crate::spec::HasTargetSpec;

fn classify_ret<'a, Ty, C>(arg: &mut ArgAbi<'a, Ty>)
where
    Ty: TyAbiInterface<'a, C> + Copy,
    C: HasDataLayout,
{
    if !arg.layout.is_sized() {
        // Not touching this...
        return;
    }

    // Preserve pointer types for LLVM's address-register return convention.
    // A pointer field in a C aggregate still uses the data-register convention.
    if let BackendRepr::Scalar(scalar) = arg.layout.backend_repr
        && matches!(scalar.primitive(), Primitive::Pointer(_))
        && (matches!(arg.layout.fields, FieldsShape::Primitive) || arg.layout.is_transparent())
    {
        return;
    }

    if !arg.layout.is_aggregate() {
        arg.extend_integer_width_to(32);
    }

    let total = arg.layout.size;
    if total.bits() > 64 {
        arg.make_indirect();
    } else if total.bits() > 32 {
        arg.cast_to(Uniform::consecutive(Reg::i32(), Size::from_bits(64)));
    } else {
        arg.cast_to(Reg::i32());
    }
}

fn classify_arg<'a, Ty, C>(arg: &mut ArgAbi<'a, Ty>)
where
    Ty: TyAbiInterface<'a, C> + Copy,
    C: HasDataLayout,
{
    if !arg.layout.is_sized() {
        // Not touching this...
        return;
    }

    // Casting a pointer to i32 here loses the EABI A4-A7 register bank.
    if let BackendRepr::Scalar(scalar) = arg.layout.backend_repr
        && matches!(scalar.primitive(), Primitive::Pointer(_))
        && (matches!(arg.layout.fields, FieldsShape::Primitive) || arg.layout.is_transparent())
    {
        return;
    }

    if !arg.layout.is_aggregate() {
        arg.extend_integer_width_to(32);
    }

    let total = arg.layout.size;
    if total.bits() > 64 {
        arg.make_indirect();
    } else if total.bits() > 32 {
        arg.cast_to(Uniform::consecutive(Reg::i32(), total));
    } else {
        arg.cast_to(Reg::i32());
    }
}

pub(crate) fn compute_abi_info<'a, Ty, C>(fn_abi: &mut FnAbi<'a, Ty>)
where
    Ty: TyAbiInterface<'a, C> + Copy,
    C: HasDataLayout + HasTargetSpec,
{
    if !fn_abi.ret.is_ignore() {
        classify_ret(&mut fn_abi.ret);
    }

    for arg in fn_abi.args.iter_mut() {
        if arg.is_ignore() {
            continue;
        }
        classify_arg(arg);
    }
}
