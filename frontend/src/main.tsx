import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "./index.css";
import App from "./App.tsx";
import { UseInkathonProvider } from "@scio-labs/use-inkathon";
import { development } from "@scio-labs/use-inkathon";

import {
  createBrowserRouter,
  createRoutesFromElements,
  Route,
  RouterProvider,
} from "react-router-dom";
import Dashboard from "@/components/Dashboard.tsx";
import CreateCourse from "./components/CreateCourse.tsx";

// const popTestnet: SubstrateChain = {
//   name: "Pop Network",
//   network: "popTestnet",
//   rpcUrls: [
//     "wss://rpc2.paseo.popnetwork.xyz",
//     "wss://rpc1.paseo.popnetwork.xyz",
//     "wss://rpc3.paseo.popnetwork.xyz",
//   ],
// };

// Create the router with routes defined
const router = createBrowserRouter(
  createRoutesFromElements(
    <>
      <Route path="/" element={<App />}>
        {/* Add more routes here as needed */}
      </Route>
      <Route path="/dashboard" element={<Dashboard />} />
      <Route path="/create" element={<CreateCourse />} />
    </>,
  ),
);

// Render the application
createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <UseInkathonProvider defaultChain={development} appName="eduverse">
      <RouterProvider router={router} />
    </UseInkathonProvider>
  </StrictMode>,
);
