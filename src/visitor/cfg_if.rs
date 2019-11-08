use syn::{
    parse::{Parse, ParseStream, Result},
    token::{Else, If},
    Attribute, Block,
};

#[derive(Debug, Clone)]
pub enum CfgExpr {
    Block(Block),
    If(CfgIf),
}

#[derive(Debug, Clone)]
pub struct CfgIf {
    pub if_token: If,
    pub cfg_attr: Attribute,
    pub then_branch: Block,
    pub else_branch: Option<(Else, Box<CfgExpr>)>,
}

impl Parse for CfgIf {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(CfgIf {
            if_token: input.parse()?,
            cfg_attr: {
                let v = input.call(Attribute::parse_outer)?;
                if v.len() == 1 {
                    v.into_iter().next().unwrap()
                } else {
                    return Err(input.error("Wrong number of attrs in cfg_if! condition"));
                }
            },
            then_branch: input.parse()?,
            else_branch: {
                let else_token: Result<Else> = input.parse();

                match else_token {
                    Ok(token) => Some((
                        token,
                        Box::new(
                            input
                                .parse::<Block>()
                                .map(CfgExpr::Block)
                                .or_else(|_| input.parse::<CfgIf>().map(CfgExpr::If))
                                .map_err(|_| input.error("Unexpected expression in else block"))?,
                        ),
                    )),
                    Err(_) => None,
                }
            },
        })
    }
}
