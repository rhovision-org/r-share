import { useEffect } from "react";
import { Route, Routes } from "react-router-dom";
import Layout from "./components/Layout";
import Admin from "./routes/Admin";
import Booking from "./routes/Booking";
import Disabled from "./routes/Disabled";
import Explore from "./routes/Explore";
import History from "./routes/History";
import Listing from "./routes/Listing";
import Me from "./routes/Me";
import Provider from "./routes/Provider";
import Settings from "./routes/Settings";
import { useWallet } from "./lib/store";

export default function App() {
  const boot = useWallet((state) => state.boot);
  useEffect(() => {
    void boot();
  }, [boot]);

  return (
    <Routes>
      <Route element={<Layout />}>
        <Route index element={<Explore />} />
        <Route path="listing/:uri" element={<Listing />} />
        <Route path="booking/:uri" element={<Booking />} />
        <Route path="provider" element={<Provider />} />
        <Route path="provider/:uri" element={<Provider />} />
        <Route path="me" element={<Me />} />
        <Route path="admin" element={<Admin />} />
        <Route path="history" element={<History />} />
        <Route path="settings" element={<Settings />} />
        <Route
          path="inbox"
          element={
            <Disabled
              title="Provider booking inbox"
              reason="Nothing on-chain enumerates bookings per provider. A guest shares the booking link; open it here to act on it."
            />
          }
        />
        <Route
          path="search"
          element={
            <Disabled
              title="Global search"
              reason="There is no indexer. Browse the curated catalogue or paste a provider or listing URI."
            />
          }
        />
        <Route
          path="ramps"
          element={
            <Disabled
              title="Fiat ramps"
              reason="A ramp is a licensed off-chain partner, not an on-chain contract. r-share has no ramp registry."
            />
          }
        />
        <Route
          path="kyc"
          element={
            <Disabled
              title="Identity checks"
              reason="Identity is off-chain and outside this primitive. The chain sees a deployer key, not a person."
            />
          }
        />
        <Route
          path="evidence"
          element={
            <Disabled
              title="Evidence uploads"
              reason="Disputes anchor a hash, not a file. Store evidence elsewhere and submit its hash."
            />
          }
        />
        <Route
          path="notifications"
          element={
            <Disabled
              title="Notifications"
              reason="The node has no websocket event stream in this client. Poll the pages or run your own indexer."
            />
          }
        />
        <Route
          path="*"
          element={<Disabled title="Not found" reason="That route does not exist." />}
        />
      </Route>
    </Routes>
  );
}
