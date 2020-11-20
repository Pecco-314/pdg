#[cfg(test)]
mod tests {
    use simple_combinators::{combinator::*, parser::*, *};
    macro_rules! assert_ok {
        ($a:expr, $b:expr) => {
            assert_eq!($a.unwrap(), $b);
        };
    }
    macro_rules! assert_err {
        ($a:expr) => {
            assert!($a.is_err())
        };
    }
    #[test]
    fn test_satisfy() {
        assert_err!(satisfy(|c| c != 'A').parse(&mut "Aa"));
        assert_ok!(satisfy(|c| c != 'A').parse(&mut "a"), 'a');
        assert_err!(satisfy(|c| c != 'A').parse(&mut ""));
    }
    #[test]
    fn test_char() {
        assert_ok!(any().parse(&mut "你好"), '你');
        assert_err!(any().parse(&mut ""));

        assert_ok!(char('H').parse(&mut "Hello"), 'H');
        assert_err!(char('H').parse(&mut "hello"));

        assert_ok!(digit().parse(&mut "123"), '1');
        assert_err!(digit().parse(&mut "FF00FF"));

        assert_ok!(alpha().parse(&mut "Angel"), 'A');
        assert_ok!(alpha().parse(&mut "angel"), 'a');
        assert_err!(alpha().parse(&mut "非拉丁字母"));

        assert_ok!(many1::<_, String>(space()).parse(&mut " \r\n\t"), " \r\n\t");

        assert_ok!(spaces().parse(&mut "  \n  \t"), ());

        assert_ok!(one_of("abc").parse(&mut "cow"), 'c');
        let s = String::from("ocean");
        assert_ok!(one_of("aeiou").parse(&mut s.as_str()), 'o');
    }
    #[test]
    fn test_string() {
        assert_ok!(string("love").parse(&mut "love"), "love");
        let mut buf = "love you";
        assert_ok!(string("love").parse(&mut buf), "love");
        assert_eq!(buf, " you");

        assert_ok!(word().parse(&mut "you are fine"), "you");
        assert_ok!(word().parse(&mut "many1 is a function"), "many");

        assert_ok!(quoted_string().parse(&mut "\"I love you\""), "I love you");
        assert_ok!(
            quoted_string().parse(&mut "\"You say \\\"I love you\\\"?\""),
            "You say \"I love you\"?"
        );
        assert_err!(quoted_string().parse(&mut "\'words in single quotes won't be parsed\'"));
    }
    #[test]
    fn test_combinators() {
        assert_ok!(char('#').with(char('a')).parse(&mut "#a"), 'a');
        assert_err!(char('#').with(char('a')).parse(&mut "a#"));
        assert_err!(char('#').with(char('a')).parse(&mut "#"));

        assert_ok!(alpha().skip(char('!')).parse(&mut "a!"), 'a');
        assert_err!(alpha().skip(char('!')).parse(&mut "1!"));
        assert_err!(alpha().skip(char('!')).parse(&mut "a?"));

        assert_ok!(digit().and(digit()).parse(&mut "12"), ('1', '2'));

        assert_ok!(
            many::<_, String>(digit())
                .between(char('{'), char('}'))
                .parse(&mut "{323}"),
            "323"
        );
    }
    #[test]
    fn test_interactive() {
        assert_ok!(
            char('(').with(digit()).skip(char(')')).parse(&mut "(1)"),
            '1'
        );
        assert_ok!(
            char('(')
                .with(digit())
                .skip(char(','))
                .and(alpha())
                .skip(char(')'))
                .parse(&mut "(1,a)"),
            ('1', 'a')
        );
    }
    #[test]
    fn test_map() {
        assert_ok!(
            satisfy(|c| c >= 'a' && c <= 'z')
                .map(|c| c.to_ascii_uppercase())
                .parse(&mut "a"),
            'A'
        );
        assert_ok!(
            digit()
                .flat_map(|c| c.to_digit(10).ok_or(ParseError))
                .parse(&mut "1"),
            1
        );
    }
    #[test]
    fn test_iter() {
        let mut v = Vec::new();
        for i in any().iter(&mut "test") {
            v.push(i);
        }
        assert_eq!(v, vec!['t', 'e', 's', 't']);
        let mut ans = 0;
        for i in digit()
            .flat_map(|c| c.to_digit(10).ok_or(ParseError))
            .iter(&mut "1234")
        {
            ans = ans * 10 + i;
        }
        assert_eq!(ans, 1234);
    }
    #[test]
    fn test_repeat() {
        assert_ok!(char('a').repeat::<String>(4).parse(&mut "aaaaa"), "aaaa");
        assert_ok!(char('a').repeat::<String>(5).parse(&mut "aaaaa"), "aaaaa");
        assert_err!(char('a').repeat::<String>(6).parse(&mut "aaaaa"));
        assert_ok!(many::<_, String>(char('a')).parse(&mut "aaaaa"), "aaaaa");
        assert_ok!(
            many(digit())
                .flat_map(|s: String| s.parse::<i32>())
                .parse(&mut "1234"),
            1234
        );
        assert_ok!(many::<_, String>(any()).parse(&mut ""), "");
        assert_err!(many1::<_, String>(any()).parse(&mut ""));

        assert_ok!(
            alpha().sep_by::<_, String>(char(',')).parse(&mut "a,a,a"),
            "aaa"
        );
        assert_ok!(
            alpha().sep_by::<_, String>(char(',')).parse(&mut "a,a,a,"),
            "aaa"
        );
        assert_err!(alpha().sep_by::<_, String>(char(',')).parse(&mut ",a,a,a"));
        assert_ok!(
            alpha().sep_by::<_, String>(char(',')).parse(&mut "a;a;a"),
            "a"
        );
    }
    #[test]
    fn test_attempt() {
        let mut buf = "ab";
        let res = attempt(char('A')).parse(&mut buf);
        assert_err!(res);
        assert_eq!(buf, "ab");
        let res = attempt(char('a')).parse(&mut buf);
        assert_ok!(res, 'a');
        assert_eq!(buf, "b");

        let mut buf = "ab";
        let res = optional(char('A')).parse(&mut buf);
        assert_ok!(res, None);
        assert_eq!(buf, "ab");
        let res = optional(char('a')).parse(&mut buf);
        assert_ok!(res, Some('a'));
        assert_eq!(buf, "b");

        let res = preview(char('B')).parse(&mut buf);
        assert_err!(res);
        assert_eq!(buf, "b");
        let res = preview(char('b')).parse(&mut buf);
        assert_ok!(res, 'b');
        assert_eq!(buf, "b");

        let res = ignore(char('B')).parse(&mut buf);
        assert_err!(res);
        assert_eq!(buf, "b");
        let res = ignore(char('b')).parse(&mut buf);
        assert_ok!(res, ());
        assert_eq!(buf, "");
    }
    #[test]
    fn test_or() {
        assert_ok!(char('a').or(char('c')).parse(&mut "candy"), 'c');
        assert_ok!(one_of("abc").or(char('d')).parse(&mut "candy"), 'c');
        assert_ok!(
            number::<usize>()
                .skip(char('!'))
                .or(number::<usize>().skip(char('?')))
                .parse(&mut "123?"),
            123
        );
    }
    #[test]
    fn test_number() {
        assert_ok!(float().parse(&mut "1.432e10"), 1.432e10);
        assert_ok!(float().parse(&mut "1.432e2"), 1.432e2);
        assert_ok!(float().parse(&mut "-1242.31"), -1242.31);

        assert_ok!(number::<i64>().parse(&mut "1.432e10"), 1.432e10 as i64);
        assert_ok!(number::<i64>().parse(&mut "1.432e2"), 1);
        assert_ok!(number::<i64>().parse(&mut "-1242.31"), -1242);

        assert_ok!(number::<usize>().parse(&mut "1.432e10"), 1.432e10 as usize);
        assert_ok!(number::<usize>().parse(&mut "1.432e2"), 1);
        assert_err!(number::<usize>().parse(&mut "-1242.31"));
    }
}
