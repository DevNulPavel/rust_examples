INSERT INTO courses (course_no, title, credits)
    VALUES ('CS301', 'Базы данных', 5);

INSERT INTO courses (course_no,  credits, title)
    VALUES ('CS305', 10, 'Анализ данных');

INSERT INTO students
    VALUES (1451, 'Анна', 2014),
           (1432, 'Виктор', 2014),
           (1556, 'Нина', 2015);

INSERT INTO exams
    VALUES (1451, 'CS301', '2016-05-25', 5),
           (1556, 'CS301', '2017-05-23', 5),
           (1451, 'CS305', '2016-05-25', 5),
           (1432, 'CS305', '2016-05-25', 4);