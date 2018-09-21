use errors::*;

use diesel;
use diesel::expression::sql_literal::sql;
use diesel::sql_types::Bool;
use diesel::prelude::*;
use models::*;
use schema::*;
use paths;
use migrations;
use worker;


pub struct Database {
    name: String,
    db: SqliteConnection,
}

impl Database {
    pub fn establish<I: Into<String>>(name: I) -> Result<Database> {
        // TODO: enforce safe name for database
        let name = name.into();

        let path = paths::data_dir()?.join(name.clone() + ".db");
        let path = path.into_os_string().into_string()
            .map_err(|_| format_err!("Failed to convert db path to utf-8"))?;

        let db = worker::spawn_fn("Connecting to database", || {
            let db = SqliteConnection::establish(&path)
                .context("Failed to connect to database")?;
            migrations::run(&db)
                .context("Failed to run migrations")?;
            Ok(db)
        }, false)?;

        Ok(Database {
            name,
            db,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn db(&self) -> &SqliteConnection {
        &self.db
    }

    /// Returns true if we didn't have this value yet
    pub fn insert_generic(&self, object: &Object) -> Result<(bool, i32)> {
        match object {
            Object::Subdomain(object) => self.insert_subdomain_struct(&NewSubdomain {
                domain_id: object.domain_id,
                value: &object.value,
            }),
            Object::IpAddr(object) => self.insert_ipaddr_struct(&NewIpAddr {
                family: &object.family,
                value: &object.value,
            }),
            Object::SubdomainIpAddr(object) => self.insert_subdomain_ipaddr_struct(&NewSubdomainIpAddr {
                subdomain_id: object.subdomain_id,
                ip_addr_id: object.ip_addr_id,
            }),
        }
    }

    pub fn insert_domain(&self, domain: &str) -> Result<()> {
        let new_domain = NewDomain {
            value: domain,
        };

        diesel::insert_into(domains::table)
            .values(&new_domain)
            .execute(&self.db)?;

        Ok(())
    }

    pub fn list_domains(&self) -> Result<Vec<Domain>> {
        use schema::domains::dsl::*;

        let results = domains.load::<Domain>(&self.db)?;

        Ok(results)
    }

    pub fn filter_domains(&self, filter: &Filter) -> Result<Vec<Domain>> {
        use schema::domains::dsl::*;

        let query = domains.filter(sql::<Bool>(filter.query()));
        let results = query.load::<Domain>(&self.db)?;

        Ok(results)
    }

    pub fn domain(&self, domain: &str) -> Result<i32> {
        use schema::domains::dsl::*;

        let domain_id = domains.filter(value.eq(&domain))
            .select(id)
            .first::<i32>(&self.db)?;

        Ok(domain_id)
    }

    pub fn domain_optional(&self, domain: &str) -> Result<Option<i32>> {
        use schema::domains::dsl::*;

        let domain_id = domains.filter(value.eq(&domain))
            .select(id)
            .first::<i32>(&self.db)
            .optional()?;

        Ok(domain_id)
    }

    /// Returns true if we didn't have this value yet
    pub fn insert_subdomain(&self, subdomain: &str, domain: &str) -> Result<(bool, i32)> {
        let domain_id = match self.domain_optional(domain)? {
            Some(domain_id) => domain_id,
            None => {
                self.insert_domain(domain)?;
                self.domain(domain)?
            },
        };

        let new_subdomain = NewSubdomain {
            domain_id,
            value: &subdomain,
        };

        self.insert_subdomain_struct(&new_subdomain)
    }

    /// Returns true if we didn't have this value yet
    pub fn insert_subdomain_struct(&self, subdomain: &NewSubdomain) -> Result<(bool, i32)> {
        // upsert is not supported by diesel

        if let Some(subdomain_id) = self.subdomain_optional(subdomain.value)? {
            // TODO: right now we don't have any fields to update
            Ok((false, subdomain_id))
        } else {
            diesel::insert_into(subdomains::table)
                .values(subdomain)
                .execute(&self.db)?;
            let id = self.subdomain(subdomain.value)?;
            Ok((true, id))
        }
    }

    pub fn list_subdomains(&self) -> Result<Vec<Subdomain>> {
        use schema::subdomains::dsl::*;

        let results = subdomains.load::<Subdomain>(&self.db)?;

        Ok(results)
    }

    pub fn filter_subdomains(&self, filter: &Filter) -> Result<Vec<Subdomain>> {
        use schema::subdomains::dsl::*;

        let query = subdomains.filter(sql::<Bool>(filter.query()));
        let results = query.load::<Subdomain>(&self.db)?;

        Ok(results)
    }

    pub fn subdomain(&self, subdomain: &str) -> Result<i32> {
        use schema::subdomains::dsl::*;

        let subdomain_id = subdomains.filter(value.eq(&subdomain))
            .select(id)
            .first::<i32>(&self.db)?;

        Ok(subdomain_id)
    }

    pub fn subdomain_optional(&self, subdomain: &str) -> Result<Option<i32>> {
        use schema::subdomains::dsl::*;

        let subdomain_id = subdomains.filter(value.eq(&subdomain))
            .select(id)
            .first::<i32>(&self.db)
            .optional()?;

        Ok(subdomain_id)
    }

    pub fn insert_ipaddr(&self, family: &str, ipaddr: &str) -> Result<(bool, i32)> {
        // TODO: maybe check if valid
        let new_ipaddr = NewIpAddr {
            family: &family,
            value: &ipaddr,
        };

        self.insert_ipaddr_struct(&new_ipaddr)
    }

    pub fn insert_ipaddr_struct(&self, ipaddr: &NewIpAddr) -> Result<(bool, i32)> {
        // upsert is not supported by diesel

        if let Some(ipaddr_id) = self.ipaddr_optional(ipaddr.value)? {
            // TODO: right now we don't have any fields to update
            Ok((false, ipaddr_id))
        } else {
            diesel::insert_into(ipaddrs::table)
                .values(ipaddr)
                .execute(&self.db)?;
            let id = self.ipaddr(ipaddr.value)?;
            Ok((true, id))
        }
    }

    pub fn list_ipaddrs(&self) -> Result<Vec<IpAddr>> {
        use schema::ipaddrs::dsl::*;

        let results = ipaddrs.load::<IpAddr>(&self.db)?;

        Ok(results)
    }

    pub fn filter_ipaddrs(&self, filter: &Filter) -> Result<Vec<IpAddr>> {
        use schema::ipaddrs::dsl::*;

        let query = ipaddrs.filter(sql::<Bool>(filter.query()));
        let results = query.load::<IpAddr>(&self.db)?;

        Ok(results)
    }

    pub fn ipaddr(&self, ipaddr: &str) -> Result<i32> {
        use schema::ipaddrs::dsl::*;

        let ipaddr_id = ipaddrs.filter(value.eq(&ipaddr))
            .select(id)
            .first::<i32>(&self.db)?;

        Ok(ipaddr_id)
    }

    pub fn ipaddr_optional(&self, ipaddr: &str) -> Result<Option<i32>> {
        use schema::ipaddrs::dsl::*;

        let ipaddr_id = ipaddrs.filter(value.eq(&ipaddr))
            .select(id)
            .first::<i32>(&self.db)
            .optional()?;

        Ok(ipaddr_id)
    }

    pub fn insert_subdomain_ipaddr(&self, subdomain_id: i32, ip_addr_id: i32) -> Result<(bool, i32)> {
        self.insert_subdomain_ipaddr_struct(&NewSubdomainIpAddr {
            subdomain_id,
            ip_addr_id,
        })
    }

    pub fn insert_subdomain_ipaddr_struct(&self, subdomain_ipaddr: &NewSubdomainIpAddr) -> Result<(bool, i32)> {
        if let Some(subdomain_ipaddr_id) = self.subdomain_ipaddr_optional(subdomain_ipaddr.subdomain_id, subdomain_ipaddr.ip_addr_id)? {
            Ok((false, subdomain_ipaddr_id))
        } else {
            diesel::insert_into(subdomain_ipaddrs::table)
                .values(subdomain_ipaddr)
                .execute(&self.db)?;
            let id = self.subdomain_ipaddr(subdomain_ipaddr.subdomain_id, subdomain_ipaddr.ip_addr_id)?;
            Ok((true, id))
        }
    }

    pub fn subdomain_ipaddr(&self, my_subdomain_id: i32, my_ip_addr_id: i32) -> Result<i32> {
        use schema::subdomain_ipaddrs::dsl::*;

        let subdomain_ipaddr_id = subdomain_ipaddrs.filter(subdomain_id.eq(my_subdomain_id))
                                                   .filter(ip_addr_id.eq(my_ip_addr_id))
                                                   .select(id)
                                                   .first::<i32>(&self.db)?;

        Ok(subdomain_ipaddr_id)
    }

    pub fn subdomain_ipaddr_optional(&self, my_subdomain_id: i32, my_ip_addr_id: i32) -> Result<Option<i32>> {
        use schema::subdomain_ipaddrs::dsl::*;

        let subdomain_ipaddr_id = subdomain_ipaddrs.filter(subdomain_id.eq(my_subdomain_id))
                                                   .filter(ip_addr_id.eq(my_ip_addr_id))
                                                   .select(id)
                                                   .first::<i32>(&self.db)
                                                   .optional()?;

        Ok(subdomain_ipaddr_id)
    }
}

#[derive(Debug, PartialEq)]
pub struct Filter {
    query: String,
}

impl Filter {
    pub fn new<I: Into<String>>(query: I) -> Filter {
        Filter {
            query: query.into(),
        }
    }

    pub fn parse(mut args: &[String]) -> Result<Filter> {
        debug!("Parsing query: {:?}", args);

        if args.is_empty() {
            return Ok(Filter::new("1"));
        }

        if args[0].to_lowercase() == "where" {
            args = &args[1..];
        } else {
            bail!("Filter must begin with WHERE");
        }

        let mut query = String::new();

        let mut expect_value = false;

        for arg in args {
            if let Some(idx) = arg.find("=") {
                if idx != 0 {
                    let (key, value) = arg.split_at(idx);
                    query += &format!(" {} = {:?}", key, &value[1..]);
                    continue;
                }
            }

            if expect_value {
                query += &format!(" {:?}", arg);
                expect_value = false;
            } else {
                if ["=", "!=", "like"].contains(&arg.to_lowercase().as_str()) {
                    expect_value = true;
                }

                query += &format!(" {}", arg);
            }
        }
        debug!("Parsed query: {:?}", query);

        Ok(Filter::new(query))
    }

    pub fn query(&self) -> &str {
        &self.query
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_simple() {
        let filter = Filter::parse(&["where".to_string(),
                                     "value=1".to_string(),
                                    ]).unwrap();
        assert_eq!(filter, Filter::new(" value = \"1\""));
    }

    #[test]
    fn test_filter_str1() {
        let filter = Filter::parse(&["where".to_string(),
                                     "value=abc".to_string(),
                                    ]).unwrap();
        assert_eq!(filter, Filter::new(" value = \"abc\""));
    }

    #[test]
    fn test_filter_str2() {
        let filter = Filter::parse(&["where".to_string(),
                                     "value".to_string(),
                                     "=".to_string(),
                                     "asdf".to_string(),
                                    ]).unwrap();
        assert_eq!(filter, Filter::new(" value = \"asdf\""));
    }

    #[test]
    fn test_filter_and() {
        let filter = Filter::parse(&["where".to_string(),
                                     "value".to_string(),
                                     "=".to_string(),
                                     "foobar".to_string(),
                                     "and".to_string(),
                                     "id".to_string(),
                                     "=".to_string(),
                                     "1".to_string(),
                                    ]).unwrap();
        assert_eq!(filter, Filter::new(" value = \"foobar\" and id = \"1\""));
    }

    #[test]
    fn test_filter_like() {
        let filter = Filter::parse(&["where".to_string(),
                                     "value".to_string(),
                                     "like".to_string(),
                                     "%foobar".to_string(),
                                    ]).unwrap();
        assert_eq!(filter, Filter::new(" value like \"%foobar\""));
    }
}
