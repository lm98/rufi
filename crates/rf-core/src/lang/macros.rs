#[macro_export]
macro_rules! lift {
    ($x:expr) => {{
        |vm| (vm, $x)
    }};
}

#[macro_export]
macro_rules! nbr {
    ($x:expr) => {{
        |vm| nbr(vm, $x)
    }};
}

#[macro_export]
macro_rules! rep {
    ($init:expr, $fun:expr) => {{
        |vm| rep(vm, $init, $fun)
    }};
}

#[macro_export]
macro_rules! foldhood {
    ($init:expr, $aggr:expr, $expr:expr) => {{
        |vm| foldhood(vm, $init, $aggr, $expr)
    }};
}

#[macro_export]
macro_rules! foldhood_plus {
    ($init:expr, $aggr:expr, $expr:expr) => {{
        |vm| foldhood_plus(vm, $init, $aggr, $expr)
    }};
}

#[macro_export]
macro_rules! mux {
    ($cond:expr, $th:expr, $el:expr) => {{
        |vm| mux(vm, $cond, $th, $el)
    }};
}

#[macro_export]
macro_rules! mid {
    () => {{
        |vm| mid(vm)
    }};
}
