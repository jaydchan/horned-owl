#![allow(dead_code)]

use bimap::BiMap;
use std::collections::HashSet;

static mut COUNTER: usize = 0;

pub trait Checkable{
    fn check(&self, ont: &Ontology)-> ();
}

#[derive(Eq,PartialEq,Hash,Copy,Clone,Debug)]
pub struct IRI(usize);

impl Checkable for IRI{
    fn check(&self, ont: &Ontology){
        if !ont.contains_id(self.0){
            panic!("Attempt to add IRI to wrong ontology")
        }
    }
}

#[derive(Eq,PartialEq,Hash,Copy,Clone,Debug)]
pub struct Class(pub IRI);

impl Checkable for Class{
    fn check(&self, ont: &Ontology){
        if !ont.contains_id((self.0).0){
            panic!("Attempt to add class to wrong ontology");
        }
    }
}

#[derive(Eq,PartialEq,Hash,Copy,Clone,Debug)]
pub struct ObjectProperty(IRI);

impl Checkable for ObjectProperty{
    fn check(&self, ont: &Ontology){
        if !ont.contains_id((self.0).0){
            panic!("Attempt to add object property to wrong ontology");
        }
    }
}

#[derive(Eq,PartialEq,Hash,Clone,Debug)]
pub struct SubClass{
    pub superclass: ClassExpression,
    pub subclass: ClassExpression,
}

impl Checkable for SubClass{
    fn check(&self, ont: &Ontology){
        self.superclass.check(ont);
        self.subclass.check(ont);
    }
}

#[derive(Eq,PartialEq,Hash,Clone,Debug)]
pub struct Some{
    pub object_property: ObjectProperty,
    pub filler: Box<ClassExpression>
}

impl Checkable for Some{
    fn check(&self, ont:&Ontology) -> ()
    {
        self.object_property.check(ont);
        self.filler.check(ont);
    }
}

#[derive(Eq,PartialEq,Hash,Clone,Debug)]
pub struct And{
    pub operands: Vec<ClassExpression>
}

impl Checkable for And
{
    fn check(&self, ont: &Ontology) -> (){
        for i in &self.operands{
            i.check(ont);
        }
    }
}

#[derive(Eq,PartialEq,Hash,Clone,Debug)]
pub struct Or{
    pub operands: Vec<ClassExpression>
}

impl Checkable for Or
{
    fn check(&self, ont: &Ontology) -> (){
        for i in &self.operands{
            i.check(ont);
        }
    }
}

#[derive(Eq,PartialEq,Hash,Clone,Debug)]
pub struct Not{
    pub operand: ClassExpression
}

impl Checkable for Not
{
    fn check(&self, ont:&Ontology) -> (){
        self.operand.check(ont)
    }
}

#[derive(Eq,PartialEq,Hash,Clone,Debug)]
pub enum ClassExpression
{
    Class(Class),
    Some(Some),
    And(And),
    Or(And),
}

impl Checkable for ClassExpression{
    fn check(&self, ont:&Ontology) -> ()
    {
        match self{
            &ClassExpression::Class(ref i) => i.check(ont),
            &ClassExpression::Some(ref i)  => i.check(ont),
            &ClassExpression::And(ref i) => i.check(ont),
            &ClassExpression::Or(ref i) => i.check(ont)
        }
    }
}

#[derive(Debug)]
pub struct OntologyID{
    pub iri: Option<IRI>,
    pub viri: Option<IRI>,
}

#[derive(Debug)]
pub struct Ontology
{
    id_str: BiMap<usize,String>,
    pub id: OntologyID,
    pub class: HashSet<Class>,
    pub subclass: HashSet<SubClass>,
    pub object_property: HashSet<ObjectProperty>,
    pub some: HashSet<ClassExpression>,
    pub and: HashSet<And>
}

impl Ontology {
    pub fn new() -> Ontology{
        Ontology{
            id_str: BiMap::new(),
            id: OntologyID{iri:None,viri:None},
            class: HashSet::new(),
            subclass: HashSet::new(),
            object_property: HashSet::new(),
            some: HashSet::new(),
            and: HashSet::new(),
        }
    }

    fn next_id(&mut self) -> usize{
        unsafe{
            COUNTER = COUNTER + 1;
            COUNTER
        }
    }

    pub fn contains_id(&self, id:usize)-> bool {
        self.id_str.contains_left(&id)
    }

    pub fn contains_iri(&self, iri:String) -> bool {
        self.id_str.contains_right(&iri)
    }

    pub fn iri(&mut self, s: String) -> IRI {
        {
            let someid = self.id_str.get_by_right(&s);
            if let Some(id) = someid {
                return IRI(*id);
            }
        }

        let id = self.next_id();
        let iri = IRI(id);
        self.id_str.insert(id,s);
        iri
    }

    pub fn iri_to_str(&self, i: IRI) -> Option<&String>{
        self.id_str.get_by_left(&i.0)
    }

    pub fn class(&mut self, i: IRI) -> Class {
        let c = Class(i);
        c.check(self);

        if let Some(_) = self.class.get(&c)
        {return c;}

        self.class.insert(c);
        c
    }

    pub fn object_property(&mut self, i: IRI) -> ObjectProperty{
        let o = ObjectProperty(i);
        o.check(self);

        if let Some(_) = self.object_property.get(&o)
        {return o;};

        self.object_property.insert(o);
        o
    }

    pub fn subclass(&mut self, superclass:Class, subclass: Class)
                    -> SubClass
    {
        self.subclass_exp(ClassExpression::Class(superclass),
                          ClassExpression::Class(subclass))
    }

    pub fn subclass_exp(&mut self, superclass:ClassExpression,
                        subclass: ClassExpression) -> SubClass
    {
        let sc = SubClass{superclass:superclass,subclass:subclass};
        sc.check(self);

        if let Some(_) = self.subclass.get(&sc)
        {return sc;}

        self.subclass.insert(sc.clone());
        sc
    }

    pub fn some(&mut self, object_property:ObjectProperty,
                class:Class)
                -> ClassExpression{
        self.some_exp(object_property,ClassExpression::Class(class))
    }

    pub fn some_exp(&mut self, object_property:ObjectProperty,
                    filler:ClassExpression) -> ClassExpression{
        let some =
            ClassExpression::Some(
                Some{object_property:object_property,
                     filler:Box::new(filler)});

        some.check(self);

        if let Some(_) = self.some.get(&some)
        {return some;}

        self.some.insert(some.clone());
        some
    }

    // Query Methods
    pub fn direct_subclass(&self, c: Class)
                           ->Vec<ClassExpression>{
        self.direct_subclass_exp(ClassExpression::Class(c))
    }

    pub fn direct_subclass_exp(&self, c: ClassExpression)
                           -> Vec<ClassExpression>{
        self.subclass
            .iter()
            .filter(|sc| sc.superclass == c )
            .map(|sc| sc.subclass.clone())
            .collect::<Vec<ClassExpression>>()
    }

    pub fn is_subclass(&self, superclass:Class, subclass:Class)
        -> bool{
        self.is_subclass_exp(ClassExpression::Class(superclass),
                             ClassExpression::Class(subclass))
    }

    pub fn is_subclass_exp(&self, superclass:ClassExpression,
                           subclass:ClassExpression)
                       ->bool{

        let first:Option<&SubClass> =
            self.subclass.iter()
            .filter(|&sc|
                    sc.superclass == superclass &&
                    sc.subclass == subclass)
            .next();

        match first
        {
            Some(_) => true,
            None => false
        }
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}