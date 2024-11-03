#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod eduverse {
    use ink::prelude::vec::Vec;
    use ink::storage::Mapping;

    #[ink(storage)]
    pub struct Eduverse {
        /// Stores a single `bool` value on the storage.
        /// Course counter for generating course IDs
        course_counter: u32,
        /// Mapping of course ID to course details
        courses: Mapping<u32, Course>,
        /// Mapping of student address to their enrollments
        student_enrollments: Mapping<AccountId, Vec<u32>>,
        /// Mapping of teacher address to their courses
        teacher_courses: Mapping<AccountId, Vec<u32>>,
        /// Mapping of course ID to enrolled students
        course_students: Mapping<u32, Vec<AccountId>>,
    }

    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub struct Course {
        id: u32,
        teacher: AccountId,
        title: Vec<u8>,
        description: Vec<u8>,
        max_students: u32,
        enrolled_count: u32,
        start_time: Timestamp,
        end_time: Timestamp,
        price: Balance,
        active: bool,
        metadata_hash: Vec<u8>,
    }

    #[ink(event)]
    pub struct CourseCreated {
        #[ink(topic)]
        course_id: u32,
        #[ink(topic)]
        teacher: AccountId,
        title: Vec<u8>,
    }

    #[ink(event)]
    pub struct StudentEnrolled {
        #[ink(topic)]
        course_id: u32,
        #[ink(topic)]
        student: AccountId,
    }
    /// Custom errors
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub enum Error {
        CourseNotFound,
        CourseIsFull,
        AlreadyEnrolled,
        NotEnrolled,
        InsufficientPayment,
        Unauthorized,
        CourseNotActive,
        InvalidTime,
    }

    impl Eduverse {
        #[ink(constructor)]
        pub fn new() -> Self {
            Eduverse {
                course_counter: 0,
                courses: Mapping::default(),
                student_enrollments: Mapping::default(),
                teacher_courses: Mapping::default(),
                course_students: Mapping::default(),
            }
        }

        #[ink(message)]
        pub fn create_course(
            &mut self,
            title: Vec<u8>,
            description: Vec<u8>,
            max_students: u32,
            start_time: Timestamp,
            end_time: Timestamp,
            price: Balance,
            metadata_hash: Vec<u8>,
        ) -> Result<u32, Error> {
            let caller = self.env().caller();
            let current_time = self.env().block_timestamp();

            // Validate times
            if start_time <= current_time || end_time <= start_time {
                return Err(Error::InvalidTime);
            }

            let course_id = self.course_counter;
            self.course_counter = self.course_counter.checked_add(1).unwrap();

            let course = Course {
                id: course_id,
                teacher: caller,
                title: title.clone(),
                description,
                max_students,
                enrolled_count: 0,
                start_time,
                end_time,
                price,
                active: true,
                metadata_hash,
            };

            // Store course
            self.courses.insert(course_id, &course);

            // Add to teacher's courses
            let mut teacher_courses = self.teacher_courses.get(caller).unwrap_or_default();
            teacher_courses.push(course_id);
            self.teacher_courses.insert(caller, &teacher_courses);

            // Emit event
            self.env().emit_event(CourseCreated {
                course_id,
                teacher: caller,
                title,
            });

            Ok(course_id)
        }
        // buy course
        #[ink(message, payable)]
        pub fn enroll(&mut self, course_id: u32) -> Result<(), Error> {
            let caller = self.env().caller();
            let course = self.courses.get(course_id).ok_or(Error::CourseNotFound)?;

            if !course.active {
                return Err(Error::CourseNotActive);
            }
            if course.enrolled_count >= course.max_students {
                return Err(Error::CourseIsFull);
            }
            if self.verify_enrollment(caller, course_id) {
                return Err(Error::AlreadyEnrolled);
            }
            if self.env().transferred_value() < course.price {
                return Err(Error::InsufficientPayment);
            }

            // Update enrollments
            let mut student_courses = self.student_enrollments.get(caller).unwrap_or_default();
            student_courses.push(course_id);
            self.student_enrollments.insert(caller, &student_courses);

            // Update course students
            let mut course_students = self.course_students.get(course_id).unwrap_or_default();
            course_students.push(caller);
            self.course_students.insert(course_id, &course_students);

            // Update course enrolled count
            let mut updated_course = course;
            updated_course.enrolled_count = updated_course.enrolled_count.checked_add(1).unwrap();
            self.courses.insert(course_id, &updated_course);

            // Emit event
            self.env().emit_event(StudentEnrolled {
                course_id,
                student: caller,
            });

            Ok(())
        }

        /// Verify if a student is enrolled in a course
        #[ink(message)]
        pub fn verify_enrollment(&self, student: AccountId, course_id: u32) -> bool {
            if let Some(enrolled_students) = self.course_students.get(course_id) {
                enrolled_students.contains(&student)
            } else {
                false
            }
        }

        /// Get course details
        #[ink(message)]
        pub fn get_course(&self, course_id: u32) -> Option<Course> {
            self.courses.get(course_id)
        }

        /// Get student's enrolled courses
        #[ink(message)]
        pub fn get_student_courses(&self, student: AccountId) -> Vec<u32> {
            self.student_enrollments.get(student).unwrap_or_default()
        }

        /// Get teacher's courses
        #[ink(message)]
        pub fn get_teacher_courses(&self, teacher: AccountId) -> Vec<u32> {
            self.teacher_courses.get(teacher).unwrap_or_default()
        }
    }

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        /// We test if the default constructor does its job.
        #[ink::test]
        fn default_works() {
            let contracts = Contracts::default();
            assert_eq!(contracts.get(), false);
        }

        /// We test a simple use case of our contract.
        #[ink::test]
        fn it_works() {
            let mut contracts = Contracts::new(false);
            assert_eq!(contracts.get(), false);
            contracts.flip();
            assert_eq!(contracts.get(), true);
        }
    }

    /// This is how you'd write end-to-end (E2E) or integration tests for ink! contracts.
    ///
    /// When running these you need to make sure that you:
    /// - Compile the tests with the `e2e-tests` feature flag enabled (`--features e2e-tests`)
    /// - Are running a Substrate node which contains `pallet-contracts` in the background
    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        /// A helper function used for calling contract messages.
        use ink_e2e::ContractsBackend;

        /// The End-to-End test `Result` type.
        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        /// We test that we can upload and instantiate the contract using its default constructor.
        #[ink_e2e::test]
        async fn default_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let mut constructor = ContractsRef::default();

            // When
            let contract = client
                .instantiate("contracts", &ink_e2e::alice(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let call_builder = contract.call_builder::<Contracts>();

            // Then
            let get = call_builder.get();
            let get_result = client.call(&ink_e2e::alice(), &get).dry_run().await?;
            assert!(matches!(get_result.return_value(), false));

            Ok(())
        }

        /// We test that we can read and write a value from the on-chain contract.
        #[ink_e2e::test]
        async fn it_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let mut constructor = ContractsRef::new(false);
            let contract = client
                .instantiate("contracts", &ink_e2e::bob(), &mut constructor)
                .submit()
                .await
                .expect("instantiate failed");
            let mut call_builder = contract.call_builder::<Contracts>();

            let get = call_builder.get();
            let get_result = client.call(&ink_e2e::bob(), &get).dry_run().await?;
            assert!(matches!(get_result.return_value(), false));

            // When
            let flip = call_builder.flip();
            let _flip_result = client
                .call(&ink_e2e::bob(), &flip)
                .submit()
                .await
                .expect("flip failed");

            // Then
            let get = call_builder.get();
            let get_result = client.call(&ink_e2e::bob(), &get).dry_run().await?;
            assert!(matches!(get_result.return_value(), true));

            Ok(())
        }
    }
}
