use crate::utils::Sandbox;

#[test]
fn uncommitted() {
    let env = Sandbox::init_scenario_with_target_and_default_settings("zero-stacks");
    env.setup_metadata(&[]);

    env.file(
        "file",
        r#"
items = ["ink ribbon", "old key", "green herb", "crank", "lighter"]

puts "You check the desk drawer..."
sleep 0.8

found = items.sample

if found == "green herb"
  puts "You found a #{found}."
  puts "You feel just a little better."
else
  puts "You found an #{found}." rescue puts "You found a #{found}."
  puts "Probably useful somewhere."
end

puts "\nA distant door unlocks."
"#,
    );

    env.but("_diff2")
        .assert()
        .success()
        .stdout_eq(snapbox::str![[r#"
───────────╮
 qs:d file │
───────────╯
 
@@ -1,0 +1,17 @@
────────────────
  ┊  1 │ +
  ┊  2 │ +items = ["ink ribbon", "old key", "green herb", "crank", "lighter"]
  ┊  3 │ +
  ┊  4 │ +puts "You check the desk drawer..."
  ┊  5 │ +sleep 0.8
  ┊  6 │ +
  ┊  7 │ +found = items.sample
  ┊  8 │ +
  ┊  9 │ +if found == "green herb"
  ┊ 10 │ +  puts "You found a #{found}."
  ┊ 11 │ +  puts "You feel just a little better."
  ┊ 12 │ +else
  ┊ 13 │ +  puts "You found an #{found}." rescue puts "You found a #{found}."
  ┊ 14 │ +  puts "Probably useful somewhere."
  ┊ 15 │ +end
  ┊ 16 │ +
  ┊ 17 │ +puts "/nA distant door unlocks."

"#]]);
}
