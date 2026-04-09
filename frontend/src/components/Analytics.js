import React, { useState, useEffect } from 'react';
import axios from 'axios';
import { LineChart, Line, BarChart, Bar, PieChart, Pie, Cell, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer } from 'recharts';
import { TrendUp, ChartBar, ShieldCheck, Bug, CheckCircle } from '@phosphor-icons/react';

const BACKEND_URL = process.env.REACT_APP_BACKEND_URL;
const API = `${BACKEND_URL}/api`;

const COLORS = {
  primary: '#002FA7',
  success: '#34C759',
  warning: '#FFCC00',
  danger: '#FF3B30',
  secondary: '#4B5563'
};

export const Analytics = () => {
  const [trends, setTrends] = useState(null);
  const [successRate, setSuccessRate] = useState(null);
  const [healthScores, setHealthScores] = useState(null);
  const [vulnerabilities, setVulnerabilities] = useState(null);
  const [loading, setLoading] = useState(true);
  const [period, setPeriod] = useState(30);

  useEffect(() => {
    fetchAnalytics();
  }, [period]);

  const fetchAnalytics = async () => {
    try {
      const [trendsRes, successRes, healthRes, vulnRes] = await Promise.all([
        axios.get(`${API}/analytics/trends?days=${period}`),
        axios.get(`${API}/analytics/success-rate?days=${period}`),
        axios.get(`${API}/analytics/health-scores`),
        axios.get(`${API}/analytics/vulnerabilities`)
      ]);

      setTrends(trendsRes.data);
      setSuccessRate(successRes.data);
      setHealthScores(healthRes.data);
      setVulnerabilities(vulnRes.data);
    } catch (error) {
      console.error('Error fetching analytics:', error);
    } finally {
      setLoading(false);
    }
  };

  if (loading) {
    return (
      <div className="min-h-screen flex items-center justify-center" data-testid="loading-analytics">
        <div className="text-center">
          <ChartBar size={48} className="mx-auto mb-4 animate-pulse text-[#002FA7]" />
          <p className="text-sm uppercase tracking-wider text-[#4B5563]">Loading Analytics...</p>
        </div>
      </div>
    );
  }

  const gradeData = healthScores ? Object.entries(healthScores.grade_distribution).map(([grade, count]) => ({
    name: grade,
    count
  })) : [];

  const vulnData = vulnerabilities ? [
    { name: 'Critical', value: vulnerabilities.total_vulnerabilities.CRITICAL, color: COLORS.danger },
    { name: 'High', value: vulnerabilities.total_vulnerabilities.HIGH, color: COLORS.warning },
    { name: 'Medium', value: vulnerabilities.total_vulnerabilities.MEDIUM, color: COLORS.primary },
    { name: 'Low', value: vulnerabilities.total_vulnerabilities.LOW, color: COLORS.secondary }
  ] : [];

  return (
    <div className="max-w-7xl mx-auto px-6 sm:px-8 py-8" data-testid="analytics-page">
      <div className="mb-8">
        <h2 className="text-4xl sm:text-5xl font-bold tracking-tighter mb-2" style={{fontFamily: 'Chivo'}}>Analytics & Insights</h2>
        <p className="text-base text-[#4B5563]">Build trends, health scores, and vulnerability analytics</p>
      </div>

      {/* Period Selector */}
      <div className="mb-6 flex gap-2" data-testid="period-selector">
        {[7, 14, 30, 90].map((days) => (
          <button
            key={days}
            onClick={() => setPeriod(days)}
            className={`px-4 py-2 text-xs uppercase tracking-wider font-medium rounded-sm transition-all ${
              period === days
                ? 'bg-[#002FA7] text-white'
                : 'bg-white border border-black/10 text-[#4B5563] hover:border-black/30'
            }`}
            data-testid={`period-${days}`}
          >
            {days} Days
          </button>
        ))}
      </div>

      {/* Success Rate Card */}
      {successRate && (
        <div className="bg-white border border-black/10 rounded-sm p-6 mb-6" data-testid="success-rate-card">
          <div className="flex items-center justify-between mb-4">
            <div className="flex items-center gap-3">
              <div className="bg-[#34C759]/10 p-3 rounded-sm">
                <CheckCircle size={24} weight="bold" className="text-[#34C759]" />
              </div>
              <div>
                <h3 className="text-xl font-bold" style={{fontFamily: 'Chivo'}}>Build Success Rate</h3>
                <p className="text-sm text-[#4B5563]">Last {period} days</p>
              </div>
            </div>
            <div className="text-right">
              <div className="text-4xl font-bold" style={{fontFamily: 'Chivo'}}>{successRate.success_rate.toFixed(1)}%</div>
            </div>
          </div>
          <div className="grid grid-cols-3 gap-4 mt-4">
            <div className="text-center p-3 bg-[#F9FAFB] rounded-sm">
              <div className="text-2xl font-bold">{successRate.total_builds}</div>
              <div className="text-xs uppercase tracking-wider text-[#4B5563]">Total</div>
            </div>
            <div className="text-center p-3 bg-[#34C759]/5 rounded-sm">
              <div className="text-2xl font-bold text-[#34C759]">{successRate.completed}</div>
              <div className="text-xs uppercase tracking-wider text-[#4B5563]">Completed</div>
            </div>
            <div className="text-center p-3 bg-[#FF3B30]/5 rounded-sm">
              <div className="text-2xl font-bold text-[#FF3B30]">{successRate.failed}</div>
              <div className="text-xs uppercase tracking-wider text-[#4B5563]">Failed</div>
            </div>
          </div>
        </div>
      )}

      {/* Charts Grid */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-6">
        {/* Build Trends */}
        {trends && trends.trend_data.length > 0 && (
          <div className="bg-white border border-black/10 rounded-sm p-6" data-testid="build-trends-chart">
            <h3 className="text-xl font-bold mb-4" style={{fontFamily: 'Chivo'}}>Build Trends</h3>
            <ResponsiveContainer width="100%" height={300}>
              <LineChart data={trends.trend_data}>
                <CartesianGrid strokeDasharray="3 3" stroke="#E5E7EB" />
                <XAxis dataKey="date" tick={{ fontSize: 12, fontFamily: 'IBM Plex Mono' }} />
                <YAxis tick={{ fontSize: 12, fontFamily: 'IBM Plex Mono' }} />
                <Tooltip contentStyle={{ fontFamily: 'IBM Plex Mono', fontSize: 12 }} />
                <Legend wrapperStyle={{ fontFamily: 'IBM Plex Mono', fontSize: 12 }} />
                <Line type="monotone" dataKey="total" stroke={COLORS.primary} strokeWidth={2} name="Total" />
                <Line type="monotone" dataKey="completed" stroke={COLORS.success} strokeWidth={2} name="Completed" />
                <Line type="monotone" dataKey="failed" stroke={COLORS.danger} strokeWidth={2} name="Failed" />
              </LineChart>
            </ResponsiveContainer>
          </div>
        )}

        {/* Compliance Trends */}
        {trends && trends.trend_data.length > 0 && (
          <div className="bg-white border border-black/10 rounded-sm p-6" data-testid="compliance-trends-chart">
            <h3 className="text-xl font-bold mb-4" style={{fontFamily: 'Chivo'}}>Compliance Score Trends</h3>
            <ResponsiveContainer width="100%" height={300}>
              <LineChart data={trends.trend_data}>
                <CartesianGrid strokeDasharray="3 3" stroke="#E5E7EB" />
                <XAxis dataKey="date" tick={{ fontSize: 12, fontFamily: 'IBM Plex Mono' }} />
                <YAxis domain={[0, 100]} tick={{ fontSize: 12, fontFamily: 'IBM Plex Mono' }} />
                <Tooltip contentStyle={{ fontFamily: 'IBM Plex Mono', fontSize: 12 }} />
                <Legend wrapperStyle={{ fontFamily: 'IBM Plex Mono', fontSize: 12 }} />
                <Line type="monotone" dataKey="avg_compliance" stroke={COLORS.primary} strokeWidth={2} name="Avg Compliance Score" />
              </LineChart>
            </ResponsiveContainer>
          </div>
        )}

        {/* Health Score Distribution */}
        {healthScores && gradeData.some(d => d.count > 0) && (
          <div className="bg-white border border-black/10 rounded-sm p-6" data-testid="health-distribution-chart">
            <h3 className="text-xl font-bold mb-4" style={{fontFamily: 'Chivo'}}>Health Score Distribution</h3>
            <div className="flex items-center justify-between mb-4">
              <div>
                <div className="text-3xl font-bold" style={{fontFamily: 'Chivo'}}>{healthScores.average_health_score}</div>
                <div className="text-sm text-[#4B5563]">Average Health Score</div>
              </div>
            </div>
            <ResponsiveContainer width="100%" height={250}>
              <BarChart data={gradeData}>
                <CartesianGrid strokeDasharray="3 3" stroke="#E5E7EB" />
                <XAxis dataKey="name" tick={{ fontSize: 12, fontFamily: 'IBM Plex Mono' }} />
                <YAxis tick={{ fontSize: 12, fontFamily: 'IBM Plex Mono' }} />
                <Tooltip contentStyle={{ fontFamily: 'IBM Plex Mono', fontSize: 12 }} />
                <Bar dataKey="count" fill={COLORS.primary} />
              </BarChart>
            </ResponsiveContainer>
          </div>
        )}

        {/* Vulnerability Distribution */}
        {vulnerabilities && vulnData.some(d => d.value > 0) && (
          <div className="bg-white border border-black/10 rounded-sm p-6" data-testid="vulnerability-chart">
            <h3 className="text-xl font-bold mb-4" style={{fontFamily: 'Chivo'}}>Vulnerability Distribution</h3>
            <p className="text-sm text-[#4B5563] mb-4">Across {vulnerabilities.total_builds_analyzed} builds</p>
            <ResponsiveContainer width="100%" height={250}>
              <PieChart>
                <Pie
                  data={vulnData}
                  cx="50%"
                  cy="50%"
                  labelLine={false}
                  label={(entry) => `${entry.name}: ${entry.value}`}
                  outerRadius={80}
                  fill="#8884d8"
                  dataKey="value"
                >
                  {vulnData.map((entry, index) => (
                    <Cell key={`cell-${index}`} fill={entry.color} />
                  ))}
                </Pie>
                <Tooltip contentStyle={{ fontFamily: 'IBM Plex Mono', fontSize: 12 }} />
              </PieChart>
            </ResponsiveContainer>
          </div>
        )}
      </div>

      {/* Runtime Vulnerability Breakdown */}
      {vulnerabilities && Object.keys(vulnerabilities.by_runtime).length > 0 && (
        <div className="bg-white border border-black/10 rounded-sm p-6" data-testid="runtime-vulnerabilities">
          <h3 className="text-xl font-bold mb-4" style={{fontFamily: 'Chivo'}}>Vulnerabilities by Runtime</h3>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
            {Object.entries(vulnerabilities.by_runtime).map(([runtime, vulns]) => (
              <div key={runtime} className="border border-black/10 rounded-sm p-4">
                <div className="text-lg font-bold uppercase mb-3" style={{fontFamily: 'Chivo'}}>{runtime}</div>
                <div className="space-y-2 text-sm font-mono">
                  <div className="flex justify-between">
                    <span className="text-[#FF3B30]">CRITICAL:</span>
                    <span className="font-bold">{vulns.CRITICAL}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-[#FFCC00]">HIGH:</span>
                    <span className="font-bold">{vulns.HIGH}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-[#002FA7]">MEDIUM:</span>
                    <span className="font-bold">{vulns.MEDIUM}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-[#4B5563]">LOW:</span>
                    <span className="font-bold">{vulns.LOW}</span>
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
};