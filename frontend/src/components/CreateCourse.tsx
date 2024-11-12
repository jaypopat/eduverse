import React, { useState } from "react";

// Define the Course interface
interface Course {
  teacher: string;
  title: string;
  description: string;
  max_students: number;
  start_time: number;
  end_time: number;
  price: number;
  metadata_hash: string; // Keep this as metadata_hash
}

const CreateCourse: React.FC = () => {
  const [course, setCourse] = useState<Course>({
    title: "",
    teacher: "",
    description: "",
    max_students: 0,
    start_time: Date.now(),
    end_time: Date.now(),
    price: 0,
    metadata_hash: "", // Initialize this as well
  });

  // Handle input changes
  const handleChange = (
    e: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement>,
  ) => {
    const { name, value } = e.target;
    setCourse((prev) => ({
      ...prev,
      [name]: value, // Update this line to handle all fields correctly
    }));
  };

  // Handle form submission
  const handleSubmit = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();

    // Placeholder for smart contract function
    try {
      await createCourseOnBlockchain(course);
      console.log("Course created:", course);
      // Optionally reset the form or show a success message
    } catch (error) {
      console.error("Error creating course:", error);
    }
  };

  // Mock function to simulate smart contract interaction
  const createCourseOnBlockchain = async (courseId: number) => {
  };

  return (
    <form
      onSubmit={handleSubmit}
      className="max-w-lg mx-auto p-4 bg-white rounded shadow-md"
    >
      <h2 className="text-2xl font-bold mb-4">Create Course</h2>
      <div className="mb-4">
        <label className="block text-sm font-medium text-gray-700">
          Teacher:
          <input
            type="text"
            name="teacher"
            value={course.teacher}
            onChange={handleChange}
            required
            className="mt-1 block w-full border border-gray-300 rounded-md p-2"
          />
        </label>
      </div>
      <div className="mb-4">
        <label className="block text-sm font-medium text-gray-700">
          Title:
          <input
            type="text"
            name="title"
            value={course.title}
            onChange={handleChange}
            required
            className="mt-1 block w-full border border-gray-300 rounded-md p-2"
          />
        </label>
      </div>
      <div className="mb-4">
        <label className="block text-sm font-medium text-gray-700">
          Description:
          <textarea
            name="description"
            value={course.description}
            onChange={handleChange}
            required
            className="mt-1 block w-full border border-gray-300 rounded-md p-2"
          />
        </label>
      </div>
      <div className="mb-4">
        <label className="block text-sm font-medium text-gray-700">
          Max Students:
          <input
            type="number"
            name="max_students"
            value={course.max_students}
            onChange={handleChange}
            required
            className="mt-1 block w-full border border-gray-300 rounded-md p-2"
          />
        </label>
      </div>
      <div className="mb-4">
        <label className="block text-sm font-medium text-gray-700">
          Start Time:
          <input
            type="datetime-local"
            name="start_time"
            value={new Date(course.start_time).toISOString().slice(0, -8)}
            onChange={(e) =>
              setCourse({
                ...course,
                start_time: new Date(e.target.value).getTime(),
              })
            }
            required
            className="mt-1 block w-full border border-gray-300 rounded-md p-2"
          />
        </label>
      </div>
      <div className="mb-4">
        <label className="block text-sm font-medium text-gray-700">
          End Time:
          <input
            type="datetime-local"
            name="end_time"
            value={new Date(course.end_time).toISOString().slice(0, -8)}
            onChange={(e) =>
              setCourse({
                ...course,
                end_time: new Date(e.target.value).getTime(),
              })
            }
            required
            className="mt-1 block w-full border border-gray-300 rounded-md p-2"
          />
        </label>
      </div>
      <div className="mb-4">
        <label className="block text-sm font-medium text-gray-700">
          Price:
          <input
            type="number"
            name="price"
            value={course.price}
            onChange={handleChange}
            required
            className="mt-1 block w-full border border-gray-300 rounded-md p-2"
          />
        </label>
      </div>
      <div className="mb-4">
        <label className="block text-sm font-medium text-gray-700">
          Additional Metadata/Links/Image URLs:
          {/* Update the name to match metadata_hash */}
          <input
            type="text"
            name="metadata_hash" // Change here to match the Course interface property
            value={course.metadata_hash} // Update here too to bind correctly to state
            onChange={handleChange}
            required
            className="mt-1 block w-full border border-gray-300 rounded-md p-2"
          />
        </label>
      </div>

      <button
        type="submit"
        className="w-full bg-blue-600 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded transition duration-200"
      >
        Create Course
      </button>
    </form>
  );
};

export default CreateCourse;
