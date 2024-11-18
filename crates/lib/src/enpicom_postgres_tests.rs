#[cfg(test)]
mod tests {
    use sqruff_lib::core::config::FluffConfig;
    use sqruff_lib_core::dialects::init::DialectKind;
    use sqruff_lib_core::dialects::syntax::SyntaxKind;
    use sqruff_lib_core::parser::context::ParseContext;
    use sqruff_lib_core::parser::matchable::MatchableTrait;
    use sqruff_lib_core::parser::parser::Parser;
    use sqruff_lib_core::parser::segments::base::Tables;
    use sqruff_lib_core::parser::segments::test_functions::lex;
    use sqruff_lib_dialects::enpicom_postgres;

    #[test]
    fn test_dialect_enpicom_postgres_specific_segment_parses() {
        let cases = [
            (
                "SelectStatementSegment",
                "SELECT * FROM x WHERE y = ANY($z { foo::bar_baz, quux }::INTEGER[])",
            ),
            ("SelectStatementSegment", "SELECT * FROM public.user"),
            ("SelectStatementSegment", "SELECT 1, 2"),
            ("NakedIdentifierSegment", "💣anything💣"),
            ("AlterProcedureActionSegment", "SET a TO b"),
            (
                "AlterProcedureActionSegment",
                "SET LOCAL something TO 💣anything💣",
            ),
            (
                "AlterProcedureActionSegment",
                "SET LOCAL something.else TO 💣anything💣",
            ),
            ("NotEqualToSegment", "!="),
            ("NakedIdentifierSegment", "online_sales"),
            ("NakedIdentifierSegment", "test_🫸number_id🫷"),
            (
                "SelectStatementSegment",
                r#"
            SELECT *
            FROM test_🫸number_id🫷
            WHERE 'apples' = 'oranges'"#,
            ),
            ("SelectStatementSegment", "☢⚞test_dynamic(1, 2)⚟☢"),
            ("SelectStatementSegment", "SELECT ☢⚞test_dynamic(1, 2)⚟☢"),
            (
                "SelectStatementSegment",
                "
            -- name: get_by_id^
            SELECT *
            FROM public.user
            WHERE id = $id::INTEGER
            AND role != 'Machine'::public.user_role_enum",
            ),
        ];

        let dialect = enpicom_postgres::dialect();
        let config = FluffConfig::from_source(
            r#"
[sqruff]
dialect = enpicom_postgres
"#,
        );
        for (segment_ref, sql_string) in cases {
            let config = config.clone();
            let parser: Parser = (&config).into();
            let mut ctx: ParseContext = (&parser).into();

            let segment = dialect.r#ref(segment_ref);
            let mut segments = lex(&dialect, sql_string);

            if segments.last().unwrap().get_type() == SyntaxKind::EndOfFile {
                segments.pop();
            }

            let tables = Tables::default();
            let match_result = segment.match_segments(&segments, 0, &mut ctx).unwrap();
            let mut parsed = match_result.apply(&tables, DialectKind::EnpicomPostgres, &segments);

            let formatted_parsed = format!("{:#?}", parsed);

            if formatted_parsed.contains(&"< Unparsable >") {
                println!("Parsed:\n\n{}", formatted_parsed);
                panic!("Unparsable segment found");
            }

            assert_eq!(
                parsed.len(),
                1,
                "finding {segment_ref} in {sql_string}\nSegments:\n{segments:?}\nParsed:\n{formatted_parsed}"
            );

            let parsed = parsed.pop().unwrap();
            let parsed_raw = parsed.raw();
            assert_eq!(
                sql_string,
                parsed.raw(),
                "string not equal parsed {sql_string}, {parsed_raw}"
            );
        }
    }
}
