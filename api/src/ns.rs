//! Standard and custom namespaces.
//!
//! This module provides:
//! * the [`Namespace`](struct.Namespace.html) type for defining custom namespace;
//! * modules corresponding to the most common namespaces.
//!
//! # Example
//! ```
//! use sophia_api::ns::{Namespace, rdf, rdfs, xsd};
//!
//! let schema = Namespace::new("http://schema.org/").unwrap();
//! let s_name = schema.get("name").unwrap();
//! // and then, given a graph:
//! //g.insert(&s_name, &rdf::type_, &rdf::Property);
//! //g.insert(&s_name, &rdfs::range, &xsd::string);
//! ```

use crate::term::SimpleIri;
use mownstr::MownStr;
use sophia_iri::{error::*, is_valid_iri_ref, resolve::*};

/// A custom namespace.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Namespace<T>(pub(crate) T);

impl<T> Namespace<T>
where
    T: AsRef<str>,
{
    /// Build a custom namespace based on the given IRI.
    ///
    /// `iri` must be a valid IRI, otherwise this constructor returns an error.
    pub fn new(iri: T) -> Result<Namespace<T>> {
        if is_valid_iri_ref(iri.as_ref()) {
            Ok(Self(iri))
        } else {
            Err(InvalidIri(String::from(iri.as_ref())))
        }
    }

    /// Build a custom namespace, without checking the given IRI.
    ///
    /// # Pre-conditions
    /// It is the callers responsibility to ensure that `iri` is a valid IRI reference.
    pub fn new_unchecked(iri: T) -> Namespace<T> {
        Self(iri)
    }

    /// Build an IRI by appending `suffix` to this namespace.
    ///
    /// Return an error if the concatenation produces an invalid IRI.
    pub fn get<'s>(&'s self, suffix: &'s str) -> Result<SimpleIri<'s>> {
        SimpleIri::new(self.0.as_ref(), Some(suffix))
    }

    /// Maps this Namespace to another one by applying function `f`.
    pub fn map<U, F>(self, f: F) -> Namespace<U>
    where
        U: AsRef<str>,
        F: FnOnce(T) -> U,
    {
        Namespace(f(self.0))
    }

    /// Tries to map this Namespace to another one by applying function `f`.
    pub fn try_map<U, F, E>(self, f: F) -> Result<Namespace<U>, E>
    where
        U: AsRef<str>,
        F: FnOnce(T) -> Result<U, E>,
    {
        Ok(Namespace(f(self.0)?))
    }

    /// Consume this Namespace and return the inner IRI data.
    pub fn destruct(self) -> T {
        self.0
    }
}

impl<'a, 'b, T> Resolve<&'a Namespace<T>, Namespace<MownStr<'a>>> for IriParsed<'b>
where
    T: AsRef<str>,
{
    /// Resolve the IRI of the given `Namespace`.
    fn resolve(&self, other: &'a Namespace<T>) -> Namespace<MownStr<'a>> {
        let iri = other.0.as_ref();
        let resolved: MownStr = self.resolve(iri).expect("Is valid as from Namespace");
        Namespace(resolved)
    }
}

impl<T: AsRef<str>> AsRef<str> for Namespace<T> {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl<T: AsRef<str>> std::ops::Deref for Namespace<T> {
    type Target = str;

    fn deref(&self) -> &str {
        self.0.as_ref()
    }
}

/// Create a "namespace module"
/// defining a set of terms within a given IRI space.
///
/// # Tests
/// This macro also create a test module to check that all created IRIs are valid.
///
/// This allows to skip those checks at runtime, keeping the initialization of the namespace fast.
#[macro_export]
macro_rules! namespace {
    ($iri_prefix:expr, $($suffix:ident),*; $($r_id:ident, $r_sf:expr),*) => {
        /// Prefix used in this namespace.
        pub static PREFIX:&'static str = $iri_prefix;
        $(
            $crate::ns_iri!($iri_prefix, $suffix);
        )*
        $(
            $crate::ns_iri!($iri_prefix, $r_id, $r_sf);
        )*

        /// Test module for checking tha IRIs are valid
        #[cfg(test)]
        mod test_valid_iri {
            #[test]
            $(
                #[allow(non_snake_case)]
                #[test]
                fn $suffix() {
                    $crate::term::SimpleIri::new($iri_prefix, Some(stringify!($suffix))).expect(stringify!($suffix));
                }
            )*
            $(
                #[allow(non_snake_case)]
                #[test]
                fn $r_id() {
                    $crate::term::SimpleIri::new($iri_prefix, Some($r_sf)).expect($r_sf);
                }
            )*
        }
    };
    ($iri_prefix:expr, $($suffix:ident),*) => {
        namespace!($iri_prefix, $($suffix),*;);
    };
}

/// Create a term in a "namespace module".
/// In general, you should use the [`namespace!`](macro.namespace.html) macro instead.
///
/// # Safety
/// This macro is conceptually unsafe,
/// as it is never checked that the prefix IRI is a valid IRI reference.
#[macro_export]
macro_rules! ns_iri {
    ($prefix:expr, $ident:ident) => {
        $crate::ns_iri!($prefix, $ident, stringify!($ident));
    };
    ($prefix:expr, $ident:ident, $suffix:expr) => {
        /// Generated term.
        #[allow(non_upper_case_globals)]
        pub static $ident: $crate::term::SimpleIri =
            $crate::term::SimpleIri::new_unchecked($prefix, Some($suffix));
    };
}

/// The standard `rdf:` namespace.
///
/// NB: since `type` is a reserved keyword in Rust,
/// the term `rdf:type` spells `rdf::type_` (with a trailing underscore).
///
pub mod rdf {
    namespace!(
        "http://www.w3.org/1999/02/22-rdf-syntax-ns#",
        // classes
        Alt,
        Bag,
        List,
        PlainLiteral,
        Property,
        Seq,
        Statement,
        // datatypes
        HTML,
        JSON,
        langString,
        XMLLiteral,
        // properties
        direction,
        first,
        language,
        object,
        predicate,
        rest,
        subject,
        value,
        // individuals
        nil,
        // core syntax terms
        RDF,
        ID,
        Description,
        about,
        parseType,
        resource,
        li,
        nodeID,
        datatype,
        bagID,
        aboutEach,
        aboutEachPrefix;
        // 'type' is a Rust keyword, so we use 'type_' instead
        type_, "type"
    );
}

/// The standard `xsd:` namespace.
#[rustfmt::skip]
pub mod xsd {
    namespace!(
        "http://www.w3.org/2001/XMLSchema#",
        anyType,
        anySimpleType,
            duration,
            dateTime,
            time,
            date,
            gYearMonth,
            gYear,
            gMonthDay,
            gDay,
            gMonth,
            boolean,
            base64Binary,
            hexBinary,
            float,
            double,
            anyURI,
            QName,
            NOTATION,
            string,
                normalizedString,
                    token,
                        language,
                        Name,
                            NCName,
                                ID,
                                IDREF,
                                    IDREFS,
                                ENTITY,
                                    ENTITIES,
                        NMTOKEN,
                        NMTOKENS,
            decimal,
                integer,
                    nonPositiveInteger,
                        negativeInteger,
                    long,
                        int,
                            short,
                                byte,
                    nonNegativeInteger,
                        unsignedLong,
                            unsignedInt,
                                unsignedShort,
                                    unsignedByte,
                        positiveInteger
    );
}

/// The standard `rdfs:` namespace.
pub mod rdfs {
    namespace!(
        "http://www.w3.org/2000/01/rdf-schema#",
        // types
        Class,
        Container,
        ContainerMembershipProperty,
        Datatype,
        Literal,
        Resource,
        // semantic properties
        domain,
        range,
        subClassOf,
        subPropertyOf,
        // documentation properties
        comment,
        isDefinedBy,
        label,
        member,
        seeAlso
    );
}

/// The standard `xml:` namespace
pub mod xml {
    namespace!(
        "http://www.w3.org/XML/1998/namespace#",
        lang,
        space,
        base,
        id,
        // Jon Bosak
        Father
    );
}

/// The standard `owl:` namespace
pub mod owl {
    namespace!(
        "http://www.w3.org/2002/07/owl#",
        Nothing,
        Thing,
        // Classes
        AllDifferent,
        AllDisjointClasses,
        AnnotationProperty,
        Class,
        DatatypeProperty,
        FunctionalProperty,
        InverseFunctionalProperty,
        IrreflexiveProperty,
        ObjectProperty,
        SymmetricProperty,
        TransitiveProperty,
        // Properties
        allValuesFrom,
        assertionProperty,
        complementOf,
        differentFrom,
        disjointWith,
        distinctMembers,
        equivalentClass,
        equivalentProperty,
        intersectionOf,
        inverseOf,
        maxCardinality,
        maxQualifiedCardinality,
        members,
        onClass,
        oneOf,
        onProperty,
        propertyChainAxiom,
        propertyDisjointWith,
        sameAs,
        someValuesFrom,
        sourceIndividual,
        targetIndividual,
        targetValue,
        unionOf
    );
}

#[cfg(test)]
mod test {
    // Nothing really worth testing here
    use super::*;
    use std::rc::Rc;

    #[test]
    fn test_same_term() {
        let ns1 = Namespace::new("http://schema.org/").unwrap();
        let ns2 = Namespace::new(Rc::from("http://schema.org/")).unwrap();

        assert_eq!(ns1.get("name").unwrap(), ns1.get("name").unwrap());
        assert_eq!(ns2.get("name").unwrap(), ns2.get("name").unwrap());
        assert_eq!(ns1.get("name").unwrap(), ns2.get("name").unwrap());
    }

    #[test]
    fn test_different_terms() {
        let ns1 = Namespace::new("http://schema.org/").unwrap();
        assert_ne!(ns1.get("name").unwrap(), ns1.get("nam").unwrap());
    }

    #[test]
    fn test_invalid_namespace() {
        assert!(Namespace::new("http://schema.org ").is_err());
    }

    #[test]
    fn test_invalid_suffix() {
        let ns1 = Namespace::new("http://schema.org/").unwrap();
        assert!(ns1.get("name ").is_err());
    }
}
