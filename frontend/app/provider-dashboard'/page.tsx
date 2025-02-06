"use client";

import { useState } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { mockProviderDashboardData } from "@/lib/mock-data";
import { LineChart, BarChart } from "@/components/ui/chart";

export default function ProviderDashboard() {
  const [activeTab, setActiveTab] = useState("overview");

  return (
    <div className="container mx-auto px-4 py-16">
      <h1 className="text-4xl font-bold mb-8">API Provider Dashboard</h1>
      <Tabs
        value={activeTab}
        onValueChange={setActiveTab}
        className="space-y-4"
      >
        <TabsList>
          <TabsTrigger value="overview">Overview</TabsTrigger>
          <TabsTrigger value="earnings">Earnings</TabsTrigger>
          <TabsTrigger value="usage">Usage Analytics</TabsTrigger>
        </TabsList>
        <TabsContent value="overview">
          <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
            <Card>
              <CardHeader>
                <CardTitle>Total Users</CardTitle>
              </CardHeader>
              <CardContent>
                <p className="text-4xl font-bold">
                  {mockProviderDashboardData.totalUsers}
                </p>
              </CardContent>
            </Card>
            <Card>
              <CardHeader>
                <CardTitle>Total Requests</CardTitle>
              </CardHeader>
              <CardContent>
                <p className="text-4xl font-bold">
                  {mockProviderDashboardData.totalRequests}
                </p>
              </CardContent>
            </Card>
            <Card>
              <CardHeader>
                <CardTitle>Total Revenue</CardTitle>
              </CardHeader>
              <CardContent>
                <p className="text-4xl font-bold">
                  ${mockProviderDashboardData.totalRevenue.toFixed(2)}
                </p>
              </CardContent>
            </Card>
          </div>
        </TabsContent>
        <TabsContent value="earnings">
          <Card>
            <CardHeader>
              <CardTitle>Revenue Overview</CardTitle>
            </CardHeader>
            <CardContent>
              <LineChart
                data={mockProviderDashboardData.revenueData}
                index="date"
                categories={["revenue"]}
                colors={["green"]}
                valueFormatter={(value) => `$${value.toFixed(2)}`}
                yAxisWidth={60}
              />
            </CardContent>
          </Card>
        </TabsContent>
        <TabsContent value="usage">
          <Card>
            <CardHeader>
              <CardTitle>API Usage</CardTitle>
            </CardHeader>
            <CardContent>
              <BarChart
                data={mockProviderDashboardData.usageData}
                index="date"
                categories={["requests"]}
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
