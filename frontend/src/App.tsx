import { Button } from '@/components/ui/button';
import { Card, CardContent } from '@/components/ui/card';
import { GraduationCap, Users, Wallet, Globe, ArrowRight } from 'lucide-react';
import {Link} from "react-router-dom";

const LandingPage = () => {
    return (
        <div className="min-h-screen bg-gradient-to-b from-blue-50 to-white">
            {/* Hero Section */}
            <header className="container mx-auto px-4 py-16 text-center">
                <h1 className="text-5xl font-bold text-blue-900 mb-6">
                    Welcome to Eduverse
                </h1>
                <p className="text-xl text-gray-600 mb-8 max-w-2xl mx-auto">
                    Experience the future of online education where virtual meets reality.
                    Join a vibrant community of learners in our educational metaverse.
                </p>
                <div className="flex gap-4 justify-center">
                    <Button className="bg-blue-600 hover:bg-blue-700">
                        Get Started <ArrowRight className="ml-2 h-4 w-4" />
                    </Button>
                    <Button variant="outline">Learn More</Button>
                </div>
            </header>

            {/* Features Section */}
            <section className="container mx-auto px-4 py-16">
                <h2 className="text-3xl font-bold text-center mb-12 text-blue-900">
                    Why Choose Eduverse?
                </h2>
                <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-8">
                    <Card className="p-6">
                        <CardContent className="space-y-4">
                            <div className="h-12 w-12 bg-blue-100 rounded-lg flex items-center justify-center">
                                <GraduationCap className="h-6 w-6 text-blue-600" />
                            </div>
                            <h3 className="text-xl font-semibold">Interactive Learning</h3>
                            <p className="text-gray-600">
                                Engage in immersive virtual classrooms that feel just like being there in person.
                            </p>
                        </CardContent>
                    </Card>

                    <Card className="p-6">
                        <CardContent className="space-y-4">
                            <div className="h-12 w-12 bg-blue-100 rounded-lg flex items-center justify-center">
                                <Users className="h-6 w-6 text-blue-600" />
                            </div>
                            <h3 className="text-xl font-semibold">Community Driven</h3>
                            <p className="text-gray-600">
                                Connect with fellow students and educators in a vibrant virtual campus environment.
                            </p>
                        </CardContent>
                    </Card>

                    <Card className="p-6">
                        <CardContent className="space-y-4">
                            <div className="h-12 w-12 bg-blue-100 rounded-lg flex items-center justify-center">
                                <Wallet className="h-6 w-6 text-blue-600" />
                            </div>
                            <h3 className="text-xl font-semibold">Blockchain Powered</h3>
                            <p className="text-gray-600">
                                Secure course enrollment with NFT certificates and seamless wallet integration.
                            </p>
                        </CardContent>
                    </Card>
                </div>
            </section>

            {/* How It Works Section */}
            <section className="bg-blue-50 py-16">
                <div className="container mx-auto px-4">
                    <h2 className="text-3xl font-bold text-center mb-12 text-blue-900">
                        How It Works
                    </h2>
                    <div className="grid md:grid-cols-3 gap-8 text-center">
                        <div className="space-y-4">
                            <div className="bg-blue-100 h-16 w-16 rounded-full flex items-center justify-center mx-auto">
                                <span className="text-2xl font-bold text-blue-600">1</span>
                            </div>
                            <h3 className="text-xl font-semibold">Connect Wallet</h3>
                            <p className="text-gray-600">
                                Sign in securely using your Web3 wallet
                            </p>
                        </div>
                        <div className="space-y-4">
                            <div className="bg-blue-100 h-16 w-16 rounded-full flex items-center justify-center mx-auto">
                                <span className="text-2xl font-bold text-blue-600">2</span>
                            </div>
                            <h3 className="text-xl font-semibold">Enroll in Courses</h3>
                            <p className="text-gray-600">
                                Browse and enroll in courses with NFT certificates
                            </p>
                        </div>
                        <div className="space-y-4">
                            <div className="bg-blue-100 h-16 w-16 rounded-full flex items-center justify-center mx-auto">
                                <span className="text-2xl font-bold text-blue-600">3</span>
                            </div>
                            <h3 className="text-xl font-semibold">Join the Metaverse</h3>
                            <p className="text-gray-600">
                                Enter virtual classrooms and start learning
                            </p>
                        </div>
                    </div>
                </div>
            </section>

            {/* CTA Section */}
            <section className="container mx-auto px-4 py-16 text-center">
                <div className="bg-blue-600 text-white rounded-2xl p-12 max-w-4xl mx-auto">
                    <h2 className="text-3xl font-bold mb-6">
                        Ready to Transform Your Learning Experience?
                    </h2>
                    <p className="text-xl mb-8 opacity-90">
                        Join Eduverse today and experience education like never before.
                    </p>
                    <Link to={"/dashboard"}><Button size="lg" variant="secondary" className="bg-white text-blue-600 hover:bg-gray-100">
                        Launch App <Globe className="ml-2 h-5 w-5" />
                    </Button>
                    </Link>
                </div>
            </section>
        </div>
    );
};

export default LandingPage;