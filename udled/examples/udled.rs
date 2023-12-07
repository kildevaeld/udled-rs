use udled::{
    any,
    token::{Ident, LineComment, Many, MultiLineComment, Opt, Or, Str, Ws},
    Error, Input,
};

const COMMENT: Or<LineComment, MultiLineComment> = Or(LineComment, MultiLineComment);

const WS: Opt<Many<Or<Ws, Or<LineComment, MultiLineComment>>>> = Opt(Many(Or(Ws, COMMENT)));

pub struct Expr;

// udled::precedence! {
//     expr -> Expr
//     rule lhs:@ WS '=' !'=' WS rhs:@ { Ok(Expr) }
//     --
//     rule lhs:@ WS "==" WS rhs:@ { Ok(Expr) }
//     --
//     rule "200" { Ok(Expr) }
// }

fn main() -> Result<(), Error> {
    let mut input = Input::new(r#"let /* test */  "Hello, World!" ost // Line comment"#);

    println!("PEEK {:?}", input.peek("let"));

    let ret = input.parse(("let", WS, Str))?;

    println!("ret: {:#?}", ret);

    println!("PEEK {}", input.peek((WS, Ident))?);

    println!("ret: {:#?}", input.parse((Ws, Ident, WS)));

    // let let_kw = input.parse("let")?;

    // println!("LET: '{}'", &input.slice()[let_kw.start..let_kw.end]);

    // let ws = input.parse(Opt(Ws))?;

    // println!("PEEK {:?} {:?}", input.peek(Str), input.peek('"'));
    // println!("{}", input.position());

    // let str = input.parse(Str)?;

    // println!("{:?} {:?}", let_kw, str);

    // let ptr = input.parse(Opt(Ws))?;
    // println!("{:?}", ptr);

    // println!("{:?} {:?} {:?}", input.peek(Ident), input.parse(Ident), ptr);

    Ok(())
}
