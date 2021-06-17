//! Namespaces used.

/// Terms of N3's logic vocabulary.
#[allow(missing_docs)]
pub mod log {
    sophia_term::namespace!(
        "http://www.w3.org/2000/10/swap/log#",
        Chaff,
        N3Document,
        Truth,
        conclusion,
        conjunction,
        content,
        definitiveDocument,
        definitiveService,
        dtlit,
        equalTo,
        implies,
        includes,
        n3String,
        notEqualTo,
        notIncludes,
        outputString,
        parsedAsN3,
        racine,
        rawType,
        rawUri,
        semantics,
        semanticsOrError,
        uri
    );
}

/// Terms of N3's math vocabulary.
#[allow(missing_docs)]
pub mod math {
    sophia_term::namespace!(
        "http://www.w3.org/2000/10/swap/math#",
        Function,
        List,
        LogicalOperator,
        ReverseFunction,
        TwoMemberedList,
        Value,
        absoluteValue,
        atan2,
        cos,
        degrees,
        difference,
        equalTo,
        exponentiation,
        greaterThan,
        integerQuotient,
        lessThan,
        memberCount,
        negation,
        notEqualTo,
        notGreaterThan,
        notLessThan,
        product,
        quotient,
        remainder,
        rounded,
        sin,
        sinh,
        sum,
        tan,
        tanh
    );
}
