CREATE TABLE cources(
    cource_no VARCHAR(30) PRIMARY KEY,
    title TEXT NOT NULL,
    credits INTEGER
);

CREATE TABLE students(
    stud_id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    start_year INTEGER NOT NULL
);

CREATE TABLE exams(
    stud_id INTEGER REFERENCES students(stud_id),
    cource_no VARCHAR(30) REFERENCES cources(cource_no),
    exam_date DATE NULL,
    grade INTEGER NOT NULL,

    PRIMARY KEY (stud_id, cource_no, exam_date)
);