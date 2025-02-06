"use client";

import { useState } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { mockUserDashboardData } from "@/lib/mock-data";
import { LineChart } from "@/components/ui/chart";

export default function UserDashboard() {
  const [activeTab, setActiveTab] = useState("overview");

  return (
    <div className="container mx-auto px-4 py-16">
      <h1 className="text-4xl font-bold mb-8">API User Dashboard</h1>
      <Tabs
        value={activeTab}
        onValueChange={setActiveTab}
        className="space-y-4"
      >
        <TabsList>
          <TabsTrigger value="overview">Overview</TabsTrigger>
          <TabsTrigger value="subscriptions">Subscriptions</TabsTrigger>
          <TabsTrigger value="usage">Usage</TabsTrigger>
        </TabsList>
        <TabsContent value="overview">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <Card>
              <CardHeader>
                <CardTitle>Active Subscriptions</CardTitle>
              </CardHeader>
              <CardContent>
                <p className="text-4xl font-bold">
                  {mockUserDashboardData.activeSubscriptions}
                </p>
              </CardContent>
            </Card>
            <Card>
              <CardHeader>
                <CardTitle>Total API Calls</CardTitle>
              </CardHeader>
              <CardContent>
                <p className="text-4xl font-bold">
                  {mockUserDashboardData.totalAPICalls}
                </p>
              </CardContent>
            </Card>
          </div>
        </TabsContent>
        <TabsContent value="subscriptions">
          <Card>
            <CardHeader>
              <CardTitle>Active Subscriptions</CardTitle>
            </CardHeader>
            <CardContent>
              <ul className="space-y-2">
                {mockUserDashboardData.subscriptions.map((sub, index) => (
                  <li key={index} className="flex justify-between items-center">
                    <span>{sub.apiName}</span>
                    <span className="text-muted-foreground">{sub.status}</span>
                  </li>
                ))}
              </ul>
            </CardContent>
          </Card>
        </TabsContent>
        <TabsContent value="usage">
          <Card>
            <CardHeader>
              <CardTitle>API Usage</CardTitle>
            </CardHeader>
            <CardContent>
              <LineChart
                data={mockUserDashboardData.usageData}
                index="date"
                categories={["apiCalls"]}
                colors={["blue"]}
                yAxisWidth={40}
              />
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  );
}
