import { useState, useEffect } from "react";
import { web3Accounts, web3Enable } from "@polkadot/extension-dapp";
import { InjectedAccount } from "@polkadot/extension-inject/types";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { BookOpen, ChevronRight, Clock, Loader2, Users } from "lucide-react";
import {
  contractQuery,
  useContract,
  useInkathon,
  decodeOutput,
  contractTx,
} from "@scio-labs/use-inkathon";
import contractJSON from "/home/jay/Dev/personal-projects/eduverse/frontend/contract.json";

interface Course {
  id: number;
  teacher: string;
  title: string;
  description: string;
  max_students: number;
  enrolled_count: number;
  start_time: number;
  end_time: number;
  price: number;
  active: boolean;
  metadata_hash: string;
}

const Dashboard = () => {
  const [account, setAccount] = useState<InjectedAccount | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState<boolean>(false);
  const [courses, setCourses] = useState<Course[]>([]);
  const [enrolledCourses, setEnrolledCourses] = useState<Course[]>([]);
  const [loadingCourses, setLoadingCourses] = useState<boolean>(false);
  const [activeTab, setActiveTab] = useState<string>("all");
  const {api, activeSigner} = useInkathon();

  const {contract} = useContract(
      contractJSON,
      "5H9gbZrr87kaFTqbksmJBAX19oFsUYG2uNCSeD4HMon5G5ES",
  );

  useEffect(() => {
    if (account) {
      fetchCourses();
      fetchEnrolledCourses(account.address);
    }
  }, [account]);

  const connectWallet = async () => {
    setLoading(true);
    setError(null);
    try {
      const extensions = await web3Enable("Eduverse");
      if (extensions.length === 0) {
        setError("No extension found");
        return;
      }

      const allAccounts = await web3Accounts();
      if (allAccounts.length > 0) {
        setAccount(allAccounts[0]);
      } else {
        setError("No accounts found");
      }
    } catch (err) {
      console.error(err);
      setError("An error occurred while connecting the wallet");
    } finally {
      setLoading(false);
    }
  };

  const enrollInCourse = async (courseId: number, amount: number) => {
    if (!account) {
      setError("No active account found");
      return;
    }

    // Get the decimals for the token

    try {
      const result = await contractTx(api, account.address, contract, "enroll", {
        value: amount, // Use the scaled amount
      }, [courseId]);
      console.log(result);
    } catch (err) {
      console.log(err);
        setError("Failed to enroll in course. Please try again.");
      console.error(err);
    }
  };

  const fetchCourses = async () => {
    setLoadingCourses(true);
    try {
      const result = await contractQuery(
          api,
          "5H9gbZrr87kaFTqbksmJBAX19oFsUYG2uNCSeD4HMon5G5ES",
          contract,
          "get_all_courses",
      );

      if (result.result.isErr) {
        throw new Error("Failed to load courses");
      }

      const humanReadableOutput = result.output!.toHuman();
      if (Array.isArray(humanReadableOutput.Ok)) {
        const courses: Course[] = humanReadableOutput.Ok.map((course: Course) => course);
        setCourses(courses);
        console.log(courses);
      } else {
        throw new Error("Unexpected output format");
      }
    } catch (err) {
      console.error("Error fetching courses:", err);
      setError(err.message);
    } finally {
      setLoadingCourses(false);
    }
  };

  const fetchEnrolledCourses = async (address: string) => {
    setLoadingCourses(true);
    try {
      const result = await contractQuery(api, address, contract, "get_student_courses", {}, ["5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty"]);
      if (result.result.isErr) {
        throw new Error("Failed to load enrolled courses");
      }

      const humanReadableOutput = result.output!.toHuman();
      if (Array.isArray(humanReadableOutput.Ok)) {
        const courses: Course[] = humanReadableOutput.Ok;
        setEnrolledCourses(courses);
      } else {
        throw new Error("Unexpected output format");
      }
    } catch (err) {
      console.error("Error fetching enrolled courses:", err);
      setError(err.message);
    } finally {
      setLoadingCourses(false);
    }
  };

  return (
      <div className="min-h-screen bg-gray-50">
        <div className="container mx-auto px-4 py-8">
          <header className="flex justify-between items-center mb-8">
            <h1 className="text-3xl font-bold text-gray-900">Eduverse Dashboard</h1>
            {account ? (
                <div className="flex items-center gap-4">
                  <span className="text-sm text-gray-600">Connected as:</span>
                  <div className="bg-green-100 text-green-800 px-4 py-2 rounded-full text-sm">
                    {`${account.address.slice(0, 6)}...${account.address.slice(-4)}`}
                  </div>
                </div>
            ) : (
                <Button onClick={connectWallet} disabled={loading} className="flex items-center gap-2">
                  {loading && <Loader2 className="h-4 w-4 animate-spin" />}
                  {loading ? "Connecting..." : "Connect Wallet"}
                </Button>
            )}
          </header>

          {error && (
              <div className="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded mb-4">
                {error}
              </div>
          )}

          {account && (
              <div className="space-y-6">
                <div className="flex justify-between items-center">
                  <h2 className="text-2xl font-semibold text-gray-800">
                    {activeTab === "all" ? "Available Courses" : "Enrolled Courses"}
                  </h2>
                  <div className="flex space-x-4">
                    <Button variant={activeTab === "all" ? "solid" : "outline"} onClick={() => setActiveTab("all")}>
                      All Courses
                    </Button>
                    <Button variant={activeTab === "enrolled" ? "solid" : "outline"} onClick={() => setActiveTab("enrolled")}>
                      Enrolled Courses
                    </Button>
                  </div>
                </div>

                {loadingCourses ? (
                    <div className="flex justify-center py-12">
                      <Loader2 className="h-8 w-8 animate-spin text-blue-600" />
                    </div>
                ) : (
                    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                      {(activeTab === "all" ? courses : enrolledCourses).map((course) => {
                        const isEnrolled = enrolledCourses.some(enrolled => enrolled.id === course.id);
                        return (
                            <Card key={course.id} className="hover:shadow-lg transition-shadow">
                              <CardHeader>
                                <CardTitle className="text-xl">{course.title}</CardTitle>
                                <CardDescription>{course.description}</CardDescription>
                              </CardHeader>
                              <CardContent>
                                <div className="space-y-3">
                                  <div className="flex items-center gap-2 text-sm text-gray-600">
                                    <Users className="h-4 w-4" />
                                    <span>{course.max_students} students max</span>
                                  </div>
                                  <div className="flex items-center gap-2 text-sm text-gray-600">
                                    <Clock className="h-4 w-4" />
                                    <span>
                              {new Date(course.start_time * 1000).toLocaleString()} - {new Date(course.end_time * 1000).toLocaleString()}
                            </span>
                                  </div>
                                  <div className="flex items-center gap-2 text-sm text-gray-600">
                                    <BookOpen className="h-4 w-4" />
                                    <span>Price: {course.price} tokens</span>
                                  </div>
                                </div>
                              </CardContent>
                              <CardFooter>
                                <Button className="w-full" onClick={() => isEnrolled ? null : enrollInCourse(course.id,1000000000000)}>
                                  {isEnrolled ? "Join the Eduverse" : "Enroll Now"}
                                  <ChevronRight className="h-4 w-4 ml-2" />
                                </Button>
                              </CardFooter>
                            </Card>
                        );
                      })}
                    </div>
                )}
              </div>
          )}
        </div>
      </div>
  );
};

export default Dashboard;