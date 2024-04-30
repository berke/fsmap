#[derive(Debug,Clone)]
pub enum Expr<T> {
    True,
    False,
    Atom(T),
    And(Box<Expr<T>>,Box<Expr<T>>),
    Or(Box<Expr<T>>,Box<Expr<T>>),
    Diff(Box<Expr<T>>,Box<Expr<T>>),
}

impl<T> Expr<T> {
    pub fn eval<F:Fn(&T)->bool>(&self,f:&F)->bool {
	match self {
	    Self::True => true,
	    Self::False => false,
	    Self::Atom(a) => f(a),
	    Self::And(x,y) => x.eval(f) && y.eval(f),
	    Self::Or(x,y) => x.eval(f) || y.eval(f),
	    Self::Diff(x,y) => x.eval(f) && !y.eval(f)
	}
    }
}
